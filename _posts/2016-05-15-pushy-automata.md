---
layout:   post
title:    "Pushy Automata"
date:     2016-05-15
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

Welcome back! This is my second post in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Automata. I decided to do another theory post first on context-free languages, and only afterwards start on a more implementation-heavy post about implementing this kind of theory in Rust for practically useful stuff. There is of course still code in this post as well :)

I'll start with a quick refresher, but for more details read the [first post]({% post_url 2016-04-10-finite-automata %}). 

# Finite Automata refresher

Last time, we looked at deterministic and non-deterministic finite automata (DFAs and NFAs resp.), which can handle regular expressions (in fact, they are equivalent). These automata are finite state machines that only (1) take input and (2) return a binary accept/reject output. Finite automata *accept* when they end up in an accept state at the end of the input. You formally describe a finite automaton is by defining:

1. the allowable input symbols (or *alphabet* {%latex%}\Sigma{%endlatex%}), 
2. the *states* {%latex%}Q{%endlatex%}, 
3. the *state transitions* (as a finite mapping {%latex%}\delta{%endlatex%}), 
4. the *start state* {%latex%}q_0{%endlatex%} and
5. the *final* or *accept states* {%latex%}F{%endlatex%}.

DFAs require every state to have one and only one transition per input symbol. NFAs can have states that don't handle certain input symbols and states that have multiple transitions for the same input symbol. An NFA-{%latex%}\varepsilon{%endlatex%} even extends the alphabet with the empty string {%latex%}\varepsilon{%endlatex%}, so a transition doesn't have to consume input. The difference between NFAs and DFAs is not in ability (NFA execution is similar to translation to DFA + execution), but NFAs are easier to define. 

# Pushdown Automata

The non-regular language example from the previous post was: "Words in this language start with zeroes and after the zeroes are an equal number of ones". We can't count an arbitrary amount of zeroes in a finite number of states, so we can't remember how many ones we need to see at the end. Therefore, we extend our automata with a *stack*. This stack makes an automaton equivalent in power to the context-free grammar. This type of grammar is used a lot in the reference manuals of programming languages. In fact, there are many tools that allow you to write a context-free grammar and generate a parser from it!  

The finite automaton that has a stack is called a pushdown automaton (PDA). You can think of the stack as a tray dispenser that you can *push* new trays *down* on. Let's kick things off with an example PDA that recognises our non-regular language example:

{% digraph Non-regular language example %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₀ [shape=doublecircle, width=0.4];
q₃ [shape=doublecircle, width=0.4];
start -> q₀;
q₀ -> q₁ [label="ε, ε → $"];
q₁ -> q₂ [label="ε"];
q₂ -> q₃ [label="ε, $ → ε"];
q₁ -> q₁ [label="0, ε → 0"];
q₂ -> q₂ [label="1, 0 → ε"];
{% enddigraph %}

So what's happening here? The transitions now have a lot more than than just the input symbol being consumed. After the comma is the top stack symbol that's popped, and after the arrow is the new stack symbol to be pushed. If you look at {%latex%}q_1{%endlatex%}, it is taking {%latex%}0{%endlatex%}'s off the input and pushed them onto the stack. Then it takes in as many {%latex%}1{%endlatex%}'s as {%latex%}0{%endlatex%}'s, by popping a {%latex%}0{%endlatex%} off the stack for every {%latex%}1{%endlatex%} in the input.  
The outer states are only there to make sure we have a fully empty stack before we go into a final state. The {%latex%}\${%endlatex%} is usually used as an *End Of Stack* character. You can also change the definition of the PDA to already hold 1 character on the stack at the start. This is part of the definition as you find it on [Wikipedia](https://en.wikipedia.org/wiki/Pushdown_automaton#Formal_definition). 

## Determinism

The previous PDA was non-deterministic, but we can make it deterministic. I've left off the stuck state, but there should be no overlapping transitions and all transitions consume either input, stack or both. 

{% digraph Non-regular language example, deterministic %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₀ [shape=doublecircle, width=0.4];
q₃ [shape=doublecircle, width=0.4];
start -> q₀;
q₀ -> q₁ [label="0, ε → $"];
q₁ -> q₂ [label="1, 0 → ε"];
q₂ -> q₃ [label="1, $ → ε"];
q₁ -> q₁ [label="0, ε → 0"];
q₂ -> q₂ [label="1, 0 → ε"];
{% enddigraph %}

Now in general, we *cannot* change our PDAs to a deterministic version (deterministic PDAs are strictly less powerful). For example, take the language of even-length binary palindromes. This language can be recognised by the following non-deterministic PDA, but not by a deterministic one:

{% digraph Even-length binary palindromes %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₀ [shape=doublecircle, width=0.4];
q₃ [shape=doublecircle, width=0.4];
start -> q₀;
q₀ -> q₁ [label="ε, ε → $"];
q₁ -> q₂ [label="ε"];
q₂ -> q₃ [label="ε, $ → ε"];
q₁ -> q₁ [label="0, ε → 0\n1, ε → 1"];
q₂ -> q₂ [label="0, 0 → ε\n1, 1 → ε"];
{% enddigraph %}

If you think about it, it makes sense that a non-deterministic PDA is more powerful than a deterministic PDA. With the NFAs in the previous post we had just a finite amount of states we could be in while executing, and you can model that with (an exponential amount of) states in a DFA. But for a PDA, it isn't just the state that matters, but the stack as well. Since the stack isn't finite, we can't just model it in more states. 

## Code

These PDAs are a bit annoying to write as is. Epsilons for the input character mean that we're not advancing the input. We could write a direct encoding of the formal definition, but then we need to resolve epsilons at runtime. The execution would become pull-based, asking for input and the top of the stack when we need it. Somehow that feels wrong to me.  
So instead we're going to adapt our definition of PDAs, so that we can write code that's still driven by the input. Let's see if we can eliminate epsilons in the input position. There are five cases:

- Add something to the stack at the start
  - Example: {%latex%}q_0 \rightarrow q_1{%endlatex%}
  - Fix: Allow stack to start with a static bunch of symbols on it
- Add multiple things to the stack on a certain input
  - Fix: Allow pushing multiple things on the stack
- Express some non-determinism easily by doing nothing with the stack or the input
  - Example: {%latex%}q_1 \rightarrow q_2{%endlatex%}
  - Fix: This is the tradition NFA epsilon move, we can do a local powerset construction for this
- Remove multiple things from the stack on a certain input
  - Fix: Allow popping multiple things off the stack
- Remove stuff from the stack at the end of the input
  - Example: {%latex%}q_1 \rightarrow q_2{%endlatex%}
  - Fix? Not sure if there is one, so we'll just have to deal with this one

With the above fixes, we can get a PDA that will always either advance one step in the input, or at the end of the input advance on the stack. These fixes can be expressed in the original definition, so it still has the same power. Here's the new version of the PDA:

{% digraph Even-length binary palindromes, v2 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₀ [shape=doublecircle, width=0.4, label="q₀′"];
q₃ [shape=doublecircle, width=0.4, label="q₃′"];
q₁ [label="q₁′"];
q₂ [label="q₂′"];
start -> q₀ [label="$"];
q₀ -> q₁ [label="0, ε → 0\n1, ε → 1"];
q₁ -> q₂ [label="0, 0 → ε\n1, 1 → ε"];
q₂ -> q₃ [label="ε, $ → ε"];
q₁ -> q₁ [label="0, ε → 0\n1, ε → 1"];
q₂ -> q₂ [label="0, 0 → ε\n1, 1 → ε"];
{% enddigraph %}

So with that, we can go to the code. I apologise for the messier transition functions. Those don't correspond to the diagram as clearly. In retrospect this approach to the transition functions would have also worked for the other PDA, although it would be slightly less efficient (I think). 

```rust
{% include {{page.id}}/binary_palindrome/src/main.rs %}
```

I left in extra prints to observe the behaviour of the PDA:

```rust
[(0, [2])]
[(1, [2, 0])]
[(1, [2, 0, 0]), (2, [2])]
[(1, [2, 0, 0, 1])]
[(1, [2, 0, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0]), (2, [2, 0, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1]), (2, [2, 0, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0]), (2, [2, 0, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (2, [2, 0, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0]), (2, [2, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0]), (2, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (2, [2])]
[(3, [])]
The input is accepted
```

Something to note is that the amount of `(state, stack)` tuples peaks at 3 and is mostly 2. That's not so bad. But in general you can have input strings with much worse behaviour! ([Try](https://github.com/Apanatshka/Apanatshka.github.io/tree/jekyll/_includes{{page.id}}/binary_palindrome/) a long string with only zeroes for example). 

# Context-free grammars

A context-free grammar (CFG) has consists of rules which are sometimes called production rules or substitution rules. Those names are basically two ways to look at the grammar: as a way to produce 'sentences' of the language that the grammar describes, or to reduce input to check if it's part of the language.  
These rules are written with terminals (symbols from the alphabet), and sorts (or grammar variables or non-terminals). A sort is defined by one or more rules. Depending on the grammar formalism, you may see {%latex%}\leftarrow{%endlatex%}, {%latex%}\rightarrow{%endlatex%}, {%latex%}={%endlatex%} or {%latex%}::={%endlatex%} between the sort and the body of the rule. Let's look at an example:

:- | :-
{%latex%} S = 0 S 0 {%endlatex%}       | {%latex%} \text{(Rule-0)} {%endlatex%}
{%latex%} S = 1 S 1 {%endlatex%}       | {%latex%} \text{(Rule-1)} {%endlatex%}
{%latex%} S = \varepsilon {%endlatex%} | {%latex%} \text{(Rule-}\varepsilon\text{)} {%endlatex%}

This CFG describes the even-length binary palindromes that our last PDA also described. It has a single sort {%latex%}S{%endlatex%}, the *start sort* of the CFG. The zero and one are terminals, symbols from the alphabet. The epsilon is still the empty string. I've labelled the rules so I can refer to them later. 

To recognise a string, we start with the input and try to reduce part of it to a sort according to one of the rules. We keep substituting until we having only a single start sort left. This is called a *derivation*. For example:

:-: | :-
{%latex%} 0010101111010100            {%endlatex%} |
{%latex%} 00101011\varepsilon11010100 {%endlatex%} | {%latex%} \text{(}\varepsilon\text{ insertion)} {%endlatex%}
{%latex%} 00101011S11010100           {%endlatex%} | {%latex%} \text{(Rule-}\varepsilon\text{)} {%endlatex%}
{%latex%} 0010101S1010100             {%endlatex%} | {%latex%} \text{(Rule-1)} {%endlatex%}
{%latex%} 001010S010100               {%endlatex%} | {%latex%} \text{(Rule-1)} {%endlatex%}
{%latex%} 00101S10100                 {%endlatex%} | {%latex%} \text{(Rule-0)} {%endlatex%}
{%latex%} 0010S0100                   {%endlatex%} | {%latex%} \text{(Rule-1)} {%endlatex%}
{%latex%} 001S100                     {%endlatex%} | {%latex%} \text{(Rule-0)} {%endlatex%}
{%latex%} 00S00                       {%endlatex%} | {%latex%} \text{(Rule-1)} {%endlatex%}
{%latex%} 0S0                         {%endlatex%} | {%latex%} \text{(Rule-0)} {%endlatex%}
{%latex%} S                           {%endlatex%} | {%latex%} \text{(Rule-0)} {%endlatex%}

Of course the other way around also works, start with the start sort and expand sorts non-deterministically to end up with the string to recognise. 

{::comment}
## Code

Let's start with a bit of code to define context-free grammars in Rust:

```rust
{% include {{page.id}}/context_free_grammar/src/main.rs %}
```

Output:

```
S -> [[Lit("0"), Var(S), Lit("0")], [Lit("1"), Var(S), Lit("1")], [Epsilon]]
```

If you're wondering why I didn't write the code for the derivation of the binary string as shown earlier, that's because it requires 'angelic non-determinism'. That's an oracle that tells you which rule to pick when there are multiple. Those are pretty hard to come by ;)
{:/comment}

## Translation to PDA

Now CFGs are equally powerful as the PDA. That means that similar to regular expressions and NFAs/DFAs, we can translate from one to the other. Let's do the grammar to automaton, since you're more likely to write a grammar that you want to execute than the other way around. The idea is that you have a PDA with an EOS symbol and the start sort. Then you get to the 'central' state in the PDA. This state replaces the topmost sort on the stack with the *reversed* body of one of it's rules (non-deterministically of course). If the topmost thing on the stack is a terminal instead, it will match the input against the terminal and drop both. Because the rule body was pushed on the stack in reverse that works out. When the EOS symbol is found and the input is found we go to the accept state. 

Let's look at the PDA for the binary palindrome grammar:

{% digraph Even-length binary palindromes, translated from the grammar %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₁ [shape=doublecircle, width=0.4];
start -> q₀ [label="$ S"];
q₀ -> q₁ [label="ε, $ → ε\nε, $ S → ε"];
q₀ -> q₀ [label="ε, S → 0 S 0\nε, S → 1 S 1\nε, S → ε\n0, 0 →ε\n1, 1 → ε"];
{% enddigraph %}

I went for the PDA which starts with an initialised stack and can manipulate multiple things on (the top of) the stack at once. That gives a more compact PDA, and is also closer to an implementable state. Sadly this example doesn't visibly show that the bodies of the rules are reversed, because all rules in this grammar are symmetrical. 

It's interesting to see that this PDA is actually smaller in states than our hand-written one. But this one does have some more overhead because it's pushing a lot of stuff on the stack including sorts. Let's see if we can reduce that overhead a little by at least making the transitions that don't consume any input into transitions that do. For that we need to merge a rule like {%latex%}\varepsilon, S \rightarrow 0 S 0{%endlatex%} with other rules that will come afterwards which do consume input. That's {%latex%}0, 0 \rightarrow \varepsilon{%endlatex%} in this case. So combining the two rules gets us {%latex%}0, S \rightarrow 0 S{%endlatex%}, by adding the input symbol consumption and resolving the stack pop. We can do the same with the other transition that takes no input. The last rule to resolve is {%latex%}\epsilon, S \rightarrow \varepsilon{%endlatex%}. This one can be merged with {%latex%}\epsilon, S \rightarrow 0 S 0{%endlatex%} and {%latex%}0, 0 \rightarrow \varepsilon{%endlatex%} to form {%latex%}0, S \rightarrow 0{%endlatex%} and with the other two transitions to form {%latex%}1, S \rightarrow 1{%endlatex%}. 

{% digraph Even-length binary palindromes, translated from the grammar %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₁ [shape=doublecircle, width=0.4];
start -> q₀ [label="$ S"];
q₀ -> q₁ [label="ε, $ → ε\nε, $ S → ε"];
q₀ -> q₀ [label="0, S → 0 S\n1, S → 1 S\n0, S → 0\n1, S → 1\n0, 0 →ε\n1, 1 → ε"];
{% enddigraph %}

Now the sort {%latex%}S{%endlatex%} has been changed from a fairly useless overhead to a marker of "we're not halfway yet". In our hand-written PDA this was not a symbol on the stack but a different state. 

When you [implement](https://github.com/Apanatshka/Apanatshka.github.io/tree/jekyll/_includes{{page.id}}/binary_palindrome/src/grammar_based.rs) this PDA you get an output that shows that there is one redundant state that it's always in:

```rust
[(0, [2, 3]), (1, [])]
[(0, [2, 0, 3]), (0, [2, 0])]
[(0, [2, 0, 0, 3]), (0, [2, 0, 0]), (0, [2])]
[(0, [2, 0, 0, 1, 3]), (0, [2, 0, 0, 1])]
[(0, [2, 0, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0]), (0, [2, 0, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1]), (0, [2, 0, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0]), (0, [2, 0, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (0, [2, 0, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0]), (0, [2, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (0, [2])]
[(1, [])]
The input is accepted
```

This redundant state comes from the two rules that don't re-add the {%latex%}S{%endlatex%}. These rules basically try to predict at every point in the input that this was the last input symbol of the first half, which most of the time isn't going to be true. We could change them to instead predict that this was first input symbol of the second half, which can only happen when the second value on top of the stack is the same as this input symbol: {%latex%}0, 0 S \rightarrow \varepsilon{%endlatex%}.  
This rule without the overhead is just a simple combination of the old rules {%latex%}\varepsilon, S \rightarrow \varepsilon{%endlatex%} and {%latex%}0, 0 \rightarrow \varepsilon{%endlatex%}. It's only because we combined with the third rule {%latex%}\varepsilon, S \rightarrow 0 S 0{%endlatex%} that we ended up in a sub-optimal situation.[^1] At this point it's pretty clear that instead of push and popping two things of which the second is the {%latex%}S{%endlatex%}, can also be expressed as just another state. 

We're going to skip translating PDAs to CFGs, as that's a less interesting thing to do in my opinion. It shows that PDAs aren't more powerful than CFGs, but isn't used for something practical as far as I know. So---at least for me---it's enough to know that someone else has proven this property. 

## Ambiguity

The binary palindrome example has some non-determinism in there that you can't get rid of, but in the end it still has only one way to check/derive a word in the language. When you can apply multiple rules in multiple orders and still find the same word, you get into the issue of ambiguity.
 
Now in general you can have multiple sorts while in the middle of a derivation. In that case you can always pick a different order in which to substitute the sort for one of its rules and therefore change the way you derive a word. So that's not a very useful definition of ambiguity. To ignore this part of the order of derivation, we'll just arbitrarily pick an order in which you should to substitute sorts in a derivation: left-to-right. This gives you a so-called leftmost derivation. If there are still multiple left-most derivations, your CFG is ambiguous. 

Let's look a simple ambiguous grammar:

:- | :-
{%latex%} Expr = Expr + Expr {%endlatex%} | {%latex%} \text{(Addition)} {%endlatex%}
{%latex%} Expr = Expr * Expr {%endlatex%} | {%latex%} \text{(Multiplication)} {%endlatex%}
{%latex%} Expr = 0           {%endlatex%} | {%latex%} \text{(Zero)} {%endlatex%}
{%latex%} Expr = 1           {%endlatex%} | {%latex%} \text{(One)} {%endlatex%}

This is a basic arithmetic expressions grammar. And yet when you write multiple additions or multiplications, you get different possible derivation trees:

<table>
  <tbody>
    <tr>
      <td>
{% graph arithmetic expressions derivation tree 1 %}
bgcolor="transparent";
ranksep=0.2;
nodesep=0.01;
node [shape=none, height=0.3];
node [label=Expr];
Add; Mul; Zero1; Zero2; One1;
{
  rank="same";
  Node [label=0, width=0.3];
  Z1; Z2;
  O1 [label=1];
  Plus [label="+"];
  Star [label="*"];
}
Add -- Zero1; Add -- Plus [weight=10]; Add -- Mul;
Zero1 -- Z1 [weight=10];
Mul -- Zero2; Mul -- Star [weight=10]; Mul -- One1;
Zero2 -- Z2 [weight=10];
One1 -- O1 [weight=10];
edge [style=invis, len=0.02];
Z1 -- Plus -- Z2 -- Star -- O1;
{% endgraph %}
      </td>
      <td>
{% graph arithmetic expressions derivation tree 2 %}
bgcolor="transparent";
ranksep=0.2;
nodesep=0.01;
node [shape=none, height=0.3];
node [label=Expr];
Add; Mul; Zero1; Zero2; One1;
{
  rank="same";
  Node [label=0, width=0.3];
  Z1; Z2;
  O1 [label=1];
  Plus [label="+"];
  Star [label="*"];
}
Add -- Zero1; Add -- Plus [weight=10]; Add -- Zero2;
Zero1 -- Z1 [weight=10];
Zero2 -- Z2 [weight=10];
Mul -- Add; Mul -- Star [weight=10]; Mul -- One1;
One1 -- O1 [weight=10];
edge [style=invis, len=0.02];
Z1 -- Plus -- Z2 -- Star -- O1;
{% endgraph %}
      </td>
    </tr>
  </tbody>
</table>

These trees show how the derivations went from sorts to terminals. In a way, they also show an ordering, where the left one does the multiplication first and the right one does the addition first. Although this is an ambiguous grammar, it doesn't have to be. The language that it captures, arithmetic expressions, has a notion of ordering between addition and multiplication, namely that multiplication goes first. This is called precedence: multiplication takes precedence over (binds tighter than) addition. For this unambiguous language you can explicitly encode the precedence rules in the grammar to get an unambiguous grammar. 

### Inherently ambiguous

There are actually Context-Free Languages (CFLs) that are inherently ambiguous, they can only be captured by ambiguous CFGs. Here's an example of an ambiguous grammar that captures an inherently ambiguous language:

:- | :- | :- | - | :- | :- | :-
{%latex%} S    {%endlatex%} | {%latex%} = A_b C       {%endlatex%} | {%latex%} \text{(Equal-A-B)}              {%endlatex%} | | {%latex%} S    {%endlatex%} | {%latex%} = A B_c       {%endlatex%} | {%latex%} \text{(Equal-B-C)}              {%endlatex%}
{%latex%} A_b  {%endlatex%} | {%latex%} = a A_b b     {%endlatex%} | {%latex%} \text{(A-B)}                    {%endlatex%} | | {%latex%} B_c  {%endlatex%} | {%latex%} = b B_c c     {%endlatex%} | {%latex%} \text{(B-C)}                    {%endlatex%}
{%latex%} A_b  {%endlatex%} | {%latex%} = \varepsilon {%endlatex%} | {%latex%} \text{(A-B-}\varepsilon\text{)} {%endlatex%} | | {%latex%} B_c  {%endlatex%} | {%latex%} = \varepsilon {%endlatex%} | {%latex%} \text{(B-C-}\varepsilon\text{)} {%endlatex%}
{%latex%} C    {%endlatex%} | {%latex%} = c C         {%endlatex%} | {%latex%} \text{(C)}                      {%endlatex%} | | {%latex%} A    {%endlatex%} | {%latex%} = a A         {%endlatex%} | {%latex%} \text{(A)}                      {%endlatex%}
{%latex%} C    {%endlatex%} | {%latex%} = \varepsilon {%endlatex%} | {%latex%} \text{(C-}\varepsilon\text{)}   {%endlatex%} | | {%latex%} A    {%endlatex%} | {%latex%} = \varepsilon {%endlatex%} | {%latex%} \text{(A-}\varepsilon\text{)}   {%endlatex%}

This describes a language that has either (1) a number of {%latex%}a{%endlatex%}'s followed by an equal number of {%latex%}b{%endlatex%}'s followed by an arbitrary number of {%latex%}c{%endlatex%}'s, or (2) an arbitrary number of {%latex%}a{%endlatex%}'s followed by a number of {%latex%}b{%endlatex%}'s followed by an equal number of {%latex%}c{%endlatex%}'s. These two options overlap when you have an equal number of {%latex%}a{%endlatex%}'s, {%latex%}b{%endlatex%}'s and {%latex%}c{%endlatex%}'s, which results in an inherent ambiguity in this case. 

## Pumping lemma

In the [previous blog post]({% post_url 2016-04-10-finite-automata %}) I originally skipped the description of the pumping lemma for regular languages. But after some feedback on the post, I [added the description of the basic idea]({% post_url 2016-04-10-finite-automata %}#addendum). The idea is that any regular language (although also other languages) will have the property of a pumping length, where any word in the language larger than this length can be pumped up to a larger word that's still in the language. For a language with a finite number of words the pumping length is larger than the largest word in the language. For infinite languages you cannot do this, which means that there are words in the language where you can find a part of the word that you're allowed to repeat an arbitrary amount of times. This arbitrary repetition corresponds with a loop in the DFA or NFA that describes the language. 

The pumping lemma for context-free languages is similar to that of regular languages. We have a pumping length and can split words larger than the pumping length into parts. Instead three parts of which the middle can be repeated, in CFLs we split words into five parts. The second and fourth part can be repeated an arbitrary amount of times as long as they are both repeated the same number of times. This makes sense because as we've seen, we can remember a bunch of things with the stack in a PDA so we can keep two parts of a word in sync with respect to repetition. From a CFG perspective it also makes sense, because the repeated parts are basically the two terminal parts that surround a recursively defined variable (for example). 

If you want to look into this further you can look up the [wikipedia page](https://en.wikipedia.org/wiki/Pumping_lemma_for_context-free_languages), or another online resource I don't know of, or a CS book on this topic. I used "Introduction to the Theory of Computation" by Michael Sipser, which goes into detail about how to write proofs with the pumping lemma (and many other interesting things). 

# Footnotes

[^1]: Though I can't guarantee that stuff will be optimal in general with this approach. I guess the approach is pretty vaguely defined anyway. Ehhh.. whatever ¯\\\_(ツ)\_/¯
