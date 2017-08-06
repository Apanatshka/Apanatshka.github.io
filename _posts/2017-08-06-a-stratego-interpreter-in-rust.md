---
layout:   post
title:    "A Stratego interpreter in Rust"
date:     2017-08-06
category: CompSci
tags:     [stratego, interpreter, Rust]
---

At the end of my last post, three months ago by now, I promised a blog post about [the Stratego interpreter that I am writing](https://gitlab.com/Apanatshka/strs). In fact I promised it soon, which sadly became "soon". Life happened, deadlines on top of deadlines with major stress. I made it through in one piece though, so here's the long promised blog post.

I assume you're already a bit familiar with Rust, most of my blog posts use it. I'm using it for side-projects, one of which is writing a performant Stratego interpreter. By Stratego, in this case, I mean the [programming language](http://strategoxt.org). In this blog post I'll give you a look into the process of building this interpreter and all that I learnt about Rust while doing so. But I'll also go deep into the quirky semantics of Stratego, because it was (usually) fun to figure out! If you like the [Wat](https://www.destroyallsoftware.com/talks/wat) talk, I'm sure you'll enjoy some of this stuff too.

*Note:* I have a couple years of experience with Stratego and spent quite some time on this project, so I suspect some of my explanation here will skip steps that might help you understand better. So I encourage you to send me questions in the [Reddit thread](https://www.reddit.com/r/rust/) where I'll post this. 

* Table of Contents
{:toc}

# An introduction to Stratego

So what is Stratego if not the board game? [Stratego](http://strategoxt.org) is a transformation language, a language in which you write transformation rules that can then be applied to your data. Since these transformation rules do pattern matching, and data in Stratego is immutable, this already starts to feel a little bit like functional programming. The innovation in Stratego is that besides single transformation rules, is has generic *strategies* for traversing data, so you can apply rules in different orderings in different places. So that's where the name Stratego comes from, and in academic circles this is sometimes called strategic programming. 

Before we go into an example of these rules and strategies, you should know that Stratego operates on ATerms, which you can read about in my [previous blog post]({% post_url 2017-05-07-first-published-crate-aterm %}). In short ATerms describe trees, where every node has a name (constructor) and a number of children (except the leaves of course, which can also be numbers or strings). Stratego is very weakly dynamically typed, you need to define a constructor with the right amount of children to be able to use it, but the types of the children can be anything. 

```stratego
signature constructors
  Var : String -> Expr // these type names don't have meaning in Stratego
  Let : String * Expr * Expr -> Expr
  Num : Int -> Expr
  BOp : Expr * Op * Expr -> Expr
  Plus : Op
  Minus : Op

rules

  var-to-one: Var(_) -> Num(1)
  
strategies

  all-var-to-one = bottomup(try(var-to-one))
```

The signatures of constructors defined here give the feeling of the AST for a small functional programming language (modulo actual functions). Although `Var` nodes would be near the leaves of an AST, we can simply write a rule `var-to-one` that matches on any of them and transforms it into a number literal. The strategy `all-var-to-one` then tries to apply this rule over an entire tree in bottom-up (or post-order) fashion. 

# Where to start?

Stratego has a "core language", which the compiler translates everything to first. A core language is an intermediate representation that is a subset of the surface language. In a way, you could say the rest is just syntactic sugar. What a core language has over any arbitrary intermediate representation is that you can write it ourselves and feed it to the same compiler, since it's just a subset of the surface language. So if you're not exactly sure of the semantics of a piece of code, you can write a little test, use the original compiler, and see what it's supposed to do. I did this quite a lot, and in [the repository](https://gitlab.com/Apanatshka/strs) you can find a directory `example-inputs` in which I saved some of those tests. 

## Stratego Core

The idea in Stratego Core is this: You always have a current term (data) implicitly that you can operate on with strategies. Here are some strategies:

1. You can replace the current term by building a different term with `!newterm`.
2. You can pattern match on the current term to put it in a variable or unpack the structure a little further with `?pattern`.
3. The semicolon combines strategies into a sequence.
4. Curly braces create a scope in which you can define fresh variables like: `{fresha, newb: strat }`. If you put an already bound variable in a pattern match there is an equality check at that position. 
5. The guarded choice operator `cond < thenbranch + elsebranch` catches failure in `cond` and goes into either branch. Before going into the `elsebranch` after failure, the bindings from `cond` are undone. 
6. There is a let construct `let strategyname(stratarg|termarg) = strategybody in letbodystrategy end`, that allows you to define local strategies, which you can call by name. These defined strategies can take term arguments and strategy arguments. 

Here's the enum definition:

```rust
pub enum Strategy {
    Build(BuildTerm),
    Match(MatchTerm),
    Seq(Box<Strategy>, Box<Strategy>),
    Scope(Vec<String>, Box<Strategy>),
    GuardedLChoice(Box<Strategy>, Box<Strategy>, Box<Strategy>),
    Let(Vec<Def>, Box<Strategy>),
    CallT(String, Vec<Strategy>, Vec<BuildTerm>),
    ...
}
```

Beside strategy calls, there can also be primitive calls. Primitives have native implementations, and there are quite a few for Stratego that the compiler uses. There are couple more, so here's the rest of the enum with some comments on what they are for:

```rust
pub enum Strategy {
    ...
    PrimT(String, Vec<Strategy>, Vec<BuildTerm>),
    CallDynamic(String, Vec<Strategy>, Vec<BuildTerm>), // calling strategies by string name
    Fail, // fail
    Id,   // do nothing
    // generic traversals: apply the strategy on the children of the current term,
    //  build up the term with the new children coming out of the strategy call
    Some(Box<Strategy>), // apply on as many as possible, fail is it fails for all children
    One(Box<Strategy>), // stop after application succeeds on one, fail if it fails for all
    All(Box<Strategy>), // only succeed if strategy succeeds on all children
    ImportTerm(ctree::ModName), // directly load a file with an ATerm
    // (yes, this is bananas to put in here when you already have primitives
    //  for such side-effects, it's probably legacy stuff)
}
```

Of particular note are `some`, `one` and `all`. These are the primitives that allow generic traversals over a whole tree, such as `bottomup(s) = all(bottomup(s)); s` (used in the example in the intro). 

**Important note:** The language and surface syntax here is called Strategy Core. The Abstract Syntax Tree representation of that core language is what we will work with. This tree representation is called CTree, and I will probably use that term more than Stratego Core. 

# The interpreter

The interpreter is a simple recursive interpreter, so it uses the normal call stack for Stratego's call stack too. This is because I'm aiming for a fast interpreter, and I figured it would be both more effort and more overhead to manage my own call stack. We've already seen the `Strategy` enum that represents the program. We also need a "context" in which the bound and unbound variables are available. The compiler makes sure there are scopes everywhere that explicitly create fresh variables before they are used, those constructs map to push a new scope on the stack of variable scopes. The context also holds other state, like the ATerm factory (which allows us to reuse the parser in the ATerm crate while defining the exact data structure for ATerms ourselves), the state for some stateful primitives, the stack tracer (which I can probably get rid of), and primitives themselves. 

```rust
impl Strategy {
  fn eval(&self, context: &MutContext, current: ATermRef) -> Result<ATermRef> {
    ...
  }
  ...
}
```

So the interpreter function is defined on the `Strategy` enum, it takes the current term and a context object, and returns a `Result` of ATerm where the error can be `StrategyFailure` or an internal interpreter error. The context is given by immutable reference because mutable references make it too difficult to work with... I'm afraid I gave up on the borrow checker there and just put `RefCell` inside the type and called it `MutContext`. I wonder if I can change that once Non-Lexical Lifetimes land. 

## Mutable contexts, backtracking and closures

Something to notice about this mutable context is that it doesn't usually work well with things like closures and backtracking (in a guarded choice). However, in this case it is exactly what we need to "easily" recreate the freaky semantics of Stratego. Closures -- you know, functions that can access variables from when they were defined -- don't carry around an environment of bindings in Stratego. Instead they have a pointer to the part of the bindings stack that they can see, *and mutate!* A closure in Stratego can bind a unbound variable to a value, therefore mutate the context in which it is executed. If that variable was from a scope outside of a guarded choice, the variable binding is also not undone when backtracking! So there is something very strange happening there, and you can [bet](https://gitlab.com/Apanatshka/strs/commit/804f2aba4eacd4c18cbf46efd537d70979cd0c7f) that I was thoroughly stumped for while, trying to wrap my head around the behaviour. I tried a number of different things, [including a hacky transaction system](https://gitlab.com/Apanatshka/strs/commit/343e70f050d76f93cc35f041ca29e2b0b0daeb61) and [partial overlay invalidation](https://gitlab.com/Apanatshka/strs/commit/6d2973a1a39818d18c46c9a446a1722dfd5e69dd). The only way I was able to understand it was as an ad-hoc thing that grew out of operationally sticky together closures and backtracking without thinking it all the way through. Mutating the context is certainly a used feature in Stratego to pass more that one term back from a strategy, but not backtracking the binding in some cases is kinda.. wat. 

So in the end the way that I modelled this is like so: The context has a stack of scopes, which binds names to strategy closures, and names to term values. A strategy closure has the `Strategy` inside, but also an offset of the stack of scopes at time of its creation. When it executes, it splits of the newer part of the stack of scopes.  

> **Side note:** in a normal function programming language, you can return closures, which means the stack can grow smaller than the offset of the closure object, which means offsets don't usually work. But in Stratego you can only pass strategies to other strategies, not return them. So here it works.  

A guarded choice pushes a special overlay scope on the stack of scopes. This overlay doesn't have predefined fresh variables, but instead catches any writes to underlying unbound variables instead of letting them go through. If the guarded choice condition fails, we pop the overlay scope, removing all bindings that should be undone. If the guarded choice condition succeeds, we "apply" the overlay, creating the actual bindings, then discarding the overlay. 

Now the closure will sometimes pop off the overlay, when the closure was made before going into the guarded choice. The difficult thing about testing this behaviour, is that the [reference interpreter in Java](https://github.com/metaborg/strategoxt/blob/49d82ee47335339a60980104eddb78f26fc7486b/strategoxt/stratego-libraries/java-backend/java/runtime/org/strategoxt/HybridInterpreter.java) has a bug that breaks backtracking; and the compiler refuses to do zero optimisations when compiling all the way to Java or C. When a closure is inlined, suddenly the behaviour is different.

```stratego
let g = ?b; fail in g <+ (!b /* this fails if g is inlined */) end
```

## Mutable contexts and pattern matching

One danger of thinking about this interpreter mostly in terms of immutable data, except for the context, was a hard to find bug I introduced in the pattern matching code. When matching a pattern in Stratego, a variable can be already bound, in which case an equality check is done, or it can be unbound, in which case it needs to be bound. Within a pattern match a variable may occur multiple times, and can be bound by one of those occurrences and matched against in the other places. The bug in the original attempt at implementing pattern matching was that bindings were directly mutated during pattern matching. So the pattern matching code could bind a variable, then later fail on another part of the pattern, and the earlier variable remained bound. Overlay scopes were already a thing by that time, so I could reuse those to easily fix the bug. 

## Lists, explosions and dynamic type checks

You can probably imagine most of the options possible for pattern matching. Literals, variables, ATerms with patterns for children. There can also be annotations on ATerms, so those can be a matched against too. There is another pattern though. It's called the "explode pattern" and it is a form of reflection. It gives you access to the constructor of a matched term as a string, and to the children as a list. You can also implode a string and a list to a term in a build operation. 

So far, so good. But what happens when you try to explode a string? Or an integer? Or a list? Since failure is a normal thing in Stratego, and type errors aren't, it would make sense to fail to explode strings and such right? Well, in case of an integer it is actually interpreted as an "integer constructor" with zero children. In case of a string, the constructor you get is an escaped version of the string (yes, starting with `\"`). Then there's the list case...

In Stratego, if you don't import the standard library, you need to at least define the constructors `Nil/0` and `Cons/2`. This is because Stratego provides special syntax for lists like this: `[]`, `[head | tail]`, `[item1, item2]`, `[item1, item2 | tail]`. Of course in Stratego Core code you only see the basic cons-nil version. Therefore an explode of a list should give you `"Cons"` or `"Nil"` for a constructor right? 

```
$> str '![]; ?cons#(children); !cons'
[]
```

... Maybe it's a weird thing with the empty list?

```
$> str '![1, 2]; ?cons#(children); !(cons, children)'
([], [1,2])
```

Ok, I guess lists are handled specially in explosions. But at least you can just use the constructor strings as normal and still get a list right?

```
$> str '!([], "Nil"#([]))'
([], Nil)
```

wat.

So here's my theory: Legacy interacting with new features. ATerms have a notion of lists as a separate thing. In Stratego you also want to walk over lists, so it helps to see them as cons-nil lists. But they are still represented specially in memory. The interaction with reflection wasn't thought through fully, and so we can observe the difference between an ATerm constructor of `Nil()` and an ATerm empty list `[]`. 

Awkward, but perfectly reproducible behaviour. I only had the refactor lots of things, touching many files, forgetting something in three different places, spending the whole day debugging issues, I'm not resenting this at all[^list-resentment]. 

# Optimising the interpreter

At the start of July I had finally squashed all the bugs I could find and passed all tests. Then it became time to check the performance of the interpreter and start optimising it. 

The first benchmark I ran to compare my interpreter's speed with the Java based interpreter was [a program](https://gforge.inria.fr/scm/viewvc.php/rec/2015-CONVECS/STRATEGO/benchexpr10.str?view=markup) from a researcher who's working on a comparison of the performance of many rewrite systems. (Some of these programs came from the [Rewriting Engines Competition](http://fsl.cs.illinois.edu/index.php/Rewrite_Engines_Competition)). 

Anyways, the result was that the Java based interpreter, including JVM startup time, was about 4 times faster than my Rust based interpreter on this program. Granted, this is the [version of the Java based interpreter](https://github.com/metaborg/strategoxt/blob/49d82ee47335339a60980104eddb78f26fc7486b/strategoxt/stratego-libraries/java-backend/java/runtime/org/strategoxt/HybridInterpreter.java) with some performance overrides. But I would have expected that simply by implementing things in Rust would be faster, especially because the Java based interpreter uses continuation passing style to implement its own call stack. 

Here are some measurements:

```
 # Rust based interpreter
$> time ./target/release/strs benches/2015-CONVECS/benchexpr10.ctree -l libstratego-lib
result = true
 # I shortened the time output here so you don't need to scroll horizontally
9.44s user 0.05s system 99% cpu 9.515 total
```

```
 # Java based interpreter
$> time java -cp /usr/local/share/strategoxt/strategoxt/strategoxt.jar org.strategoxt.HybridInterpreter benches/2015-CONVECS/benchexpr10.ctree main
result = true
 # snipped the stacktrace here on an explicit exit with code 0, silly interpreter
6.28s user 0.35s system 279% cpu 2.372 total
```

```
 # Compiling to Java and timing the Java execution (without the JVM startup I think)
$> str -i benches/2015-CONVECS/benchexpr10.str
result = true

real	0m0.450s
user	0m0.683s
sys	0m0.085s
 # So the sum total is: 0m1.218s
```

At this point I decided it was time to study some blog posts I has bookmarked about profiling Rust programs. I settled on using valgrind with the callgrind and cachegrind tools. (Cachegrind wasn't really necessary because I never came close to needing to optimise cache misses, but I included it in the first runs of the profiler anyway). The profiler pointed out that most time was spent in malloc related functions. I *had* been taking many shortcuts by using clone instead of trying to reason with the borrow checker. Time to reduce those allocations. 

## Removing overzealous cloning [(commit)](https://gitlab.com/Apanatshka/strs/commit/3c725ab20e6a995a21340551384f5abe7b3899df)

The first place I went looking was a place where I'd tried to return a borrow, but borrowck wouldn't have it. This was the place which looks up the code for constructing a closure. Since all code is already in memory, a closure only need a borrow, it doesn't need to clone the entire tree of instructions. Of course that does require that the closures, which live on the scopes stack, have a lifetime parameter for that borrow, which means the scopes stack needs a lifetime parameter, which means the `MutContext` needs a lifetime parameter. Since `MutContext` is an argument to all the primitives I implemented, I made good use of the find-and-replace mechanism of my IDE. 

The result of this change was massive:

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr10.ctree -l libstratego-lib
result = true
1.46s user 0.02s system 99% cpu 1.496 total
```

Suddenly we're already faster than the Java based interpreter and closing in on the compiled version! Let's switch to a bigger benchmark. This is one where the Java based interpreter takes forever, so I'm excluding that one.

```
$> str -i benches/2015-CONVECS/benchexpr20.str # Compiled version
result = true

real	0m4.552s
user	0m8.194s
sys	0m0.729s
 # Sum total: 0m13.475s
```

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr20.ctree -l libstratego-lib
result = true
1298.19s user 2.96s system 99% cpu 21:43.63 total
 # This benchmark could have a slightly higher time
 # because I was doing other things while the benchmark was running
 # but the point is it takes really long for our interpreter to get through this benchmark
```

## FNV hashing [(commit)](https://gitlab.com/Apanatshka/strs/commit/cfd4c0ca48412f5e23d2db8e72b7d8a558dc1024)

One fairly easy thing we can do in Rust (I really &#10084; that design), is swap out the hash function for hashsets and hashmaps. This is significant because a scope uses hashmaps to map from variable names to values. So I tried [FNV](https://docs.rs/fnv/) hashing, and it sped things up nicely with minimal code change!

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr10.ctree -l libstratego-lib
result = true # this is the smaller benchmark. The old total time was 1.496
1.15s user 0.02s system 99% cpu 1.177 total
 # Note that this is a faster time than the compiled version!
```

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr20.ctree -l libstratego-lib
result = true # this is the larger benchmark. The old total time was 21:43.63
1000.62s user 2.65s system 99% cpu 16:45.36 total
 # This time is still 77x slower than the compiled version though
```

## Make structure follow common patterns [(commit)](https://gitlab.com/Apanatshka/strs/commit/f100043fc4cc3ba4ed06d74a7e6f941efe028bdd)

While I was figuring out the weirdness around closures and backtracking, I found a little optimisation that the Java based interpreter made to its internal CTree representation. Sequences were lists instead of right-recursive binary trees, and guarded choice were lists of pairs. These are common patterns in Stratego Core, long chains of sequences and guarded choices. Flattening those patterns in the tree representation helps quite a bit. I already had a preprocessing step on the CTree to simplify things a bit, so this fit right in. 

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr10.ctree -l libstratego-lib
result = true # this is the smaller benchmark. The old total time was 1.177
1.01s user 0.02s system 99% cpu 1.039 total
```

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr20.ctree -l libstratego-lib
result = true # this is the larger benchmark. The old total time was 16:45.36
833.91s user 0.70s system 99% cpu 13:54.76 total
```

## Specialise pattern matching [(commit)](https://gitlab.com/Apanatshka/strs/commit/96fee995511645faa9be6b20bb229af24a56c1c1)

Those long guarded choice chains, those mostly come from overloaded rules. Overloaded rules typically match on different constructors. If most are matching on different constructors, then we can find the constructor literals in the CTree as strings. So we could for the first level of the match exclude a lot of patterns by just checking the constructor of the term being matched. The addition here is a pre-selection mechanism based on constructor string, using a hashmap. Of course it *is* important to keep ordering of rules the same when multiple match on the same outer constructor, so there is a bit of thought (and code) that went into this to keep the same semantics. 

(Using commit [45bb8b56](https://gitlab.com/Apanatshka/strs/commit/45bb8b56a4439531681e923f86947e2cf95aa584) instead of the commit linked by the heading, because of bugfixes after the change)

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr10.ctree -l libstratego-lib
result = true # this is the smaller benchmark. The old total time was 1.039
0.18s user 0.02s system 96% cpu 0.204 total
```

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr20.ctree -l libstratego-lib
result = true # this is the larger benchmark. The old total time was 13:54.76
61.18s user 0.50s system 99% cpu 1:01.81 total
```

Finally getting more reasonable times! Still ~4.5x slower than the compiled version. But it can now take longer to compile the release build than to run this benchmark. 

## Less String allocation a.k.a. *all* the lifetimes! [(commit)](https://gitlab.com/Apanatshka/strs/commit/39f8cb8624d21876f32807705b898e24925b3135)

At this point I was still seeing a lot of time spent in allocation, and I figured all those owned strings for constructors in the ATerm data couldn't be very good. If we could share all those constructor strings, that would also make matching faster because you could do pointer equality for the constructors. This took a lot of changes in the [aterm](https://docs.rs/aterm/0.17.0/aterm/) crate, which brought the version up to 0.17. Then many more changes in the interpreter itself, threading a new lifetime parameter through basically every type I had defined, because they all contained ATerms which now had a lifetime for the `&str`. Kind of ugly, and I'm not really happy about how much time and effort it took. I wonder if lifetime elision can be improved, or if this is really necessary and what I'm missing is a good refactoring tool. In the end there was a nice speedup though. 

(The linked commit in the heading is the one introducing all the lifetimes. Commit [8689c23e](https://gitlab.com/Apanatshka/strs/commit/8689c23e5f34d26c3c5dda33027c549c27d1bc62) is used below for the benchmarks. )

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr10.ctree -l libstratego-lib
result = true # this is the smaller benchmark. The old total time was 0.204
0.18s user 0.02s system 96% cpu 0.206 total 
```

```
$> time ./target/release/strs benches/2015-CONVECS/benchexpr20.ctree -l libstratego-lib
result = true # this is the larger benchmark. The old total time was 1:01.81
60.04s user 0.51s system 99% cpu 1:00.60 total
```

After all that, this isn't very exciting is it? I'm not sure why this doesn't have much of an impact. 

## Lazily cache constants

(Commits [4845c86c](https://gitlab.com/Apanatshka/strs/commit/4845c86cdfbf5a6a236d4c1c0a45d7ff0e70dcc4) through [6d72c976](https://gitlab.com/Apanatshka/strs/commit/6d72c9760212d238f73e1f77ba3c14d8d88f9a63) -- I need to refactor this code, it's kind of ugly)

The compiler for Stratego extracts constants and initialises them at startup of the program. I'm not sure if that's part of the time measurement it gives. To do something similar I added another caching layer on some build and match patterns during preprocessing of the CTree, when there aren't any variables in the pattern and the pattern is "large enough". Currently it's pretty conservative and only adds the caching layer at the top level. This did not slow down or speed up the two benchmarks, and since the larger one wasn't slowed down I'm leaving it in. I still need to try if it has a positive effect with more aggressive caching of subterms. 

## Planned: Copy Propagation on CTree

After profiling and optimising the interpreter internals to the best of my abilities I was thinking about what else there is to optimise. The core code that's output by the compiler is not really optimised, optimisations are done in a later stage on the Java code. Some of those optimisations seem to coincide with what I started thinking about while reading CTree output of small example programs. 

The compiler introduces a lot of local variables that are only bound to the values of other variables and built later. Basically making a lot of useless aliases when the original variable could also be used. I'm hoping to make a tool that does a CTree &#8594; CTree transformation that removes a lot of useless variables and some of the code around that. That way it can be a preprocessing step that'll help both my interpreter and the Java based version. 

## Primitives and Perfect Hashing

One last thing I'd like to mention is an optimisation I added very near the start of the project. The list of primitives is know at compile-time, so we could make lookup of primitive strings really fast through compile-time preprocessing. I had already read about [phf](https://docs.rs/phf/0.7.21/phf/), a crate for compile-time optimized maps and sets using Perfect Hash Functions. I'm currently using phf_codegen, to avoid needing the nightly compiler for the generation of the PHF maps. This way I also learned a bit about how to use a `build.rs` script, including adapting it to [create unit tests based on a directory of input files](https://gitlab.com/Apanatshka/strs/blob/ee7a9610776508a93423e625221a6f02c44e37f4/build.rs#L11).

# Process

Originally I had a pretty good idea how much time I'd spent on this project. I spent half a day to finish the ATerm parser. Then I spent three and a half days getting the basic language constructs parsed and interpreted. This was a long weekend, so I had time to do that kind of a sprint. After a normal weekend where I didn't have much time to work on it, there was another 4 day weekend. So I spent another 3 days in concentration and implemented a lot of primitives from the standard library. After that I got less and less time, snuck in an hour or two here and there on an evening or weekend. At this point I lost track of how much time I was spending on the project exactly, but it can't have been too much with stolen hours and the ramp up time it takes to load the code base into your short-term memory again (even a fairly small one). After the deadlines were over I spent my free time on recuperating, and only after a while picked this project back up. 

## Tests

Fairly early, a week or so after the second long weekend sprint, I made the script to run all the compiler tests. It took a long time to get through all of them, but it made this whole project infinitely easier. I mean that, I don't think I could have made a correct, bug-for-bug compatible interpreter of this language without the 135 tests[^no-of-tests]. 

## Tools

I'm very happy with the new style of the nightly `rustfmt`! It looks so much better. I do run into a bug of output that doesn't parse, which I should really try to narrow down and submit an issue about. 

I also used `clippy` extensively during this project. I've been a fan of this tool from the start, and I'm particularly thankful for the [`needless_lifetimes`](https://github.com/rust-lang-nursery/rust-clippy/wiki#needless_lifetimes) lint. I never remember the lifetime elision rules, and during optimisation of the code I was adding lifetimes left and right. When I forgot some, typeck or borrowck would prod me to add them, but clippy helped me clean up and simplify code. 

I already mentioned tests. I guess it was also obvious in the post that I ran the benchmarks manually. Most of the benchmark programs simply ran too long to really have a use for the benchmark harness in nightly.

I also mentioned callgrind for profiling. I used kcachegrind as the tool to explore the results. I don't understand all of the options, but enough to explore what the biggest costs were. 

# Footnotes

[^list-resentment]: Actually if I [look back at my commit message](https://gitlab.com/Apanatshka/strs/commit/dd224f4a35bb5905b5019cdf2cd9787ce1601cb7), I truly didn't resent the change. I was already running into the issues of converting between ATerm list and cons-nil list, so according to the commit message everything became easier actually. 

[^no-of-tests]: Originally there were a few more, but some didn't make sense and the original compiler and interpreter failed them too. 
