---
layout:   post
title:    "Implementing Finite Automata (Part 2)"
date:     2016-11-12
category: CompSci
tags:     [theory, automata, computation, finite automata, nfa, dfa, powerset construction, regular language, rust]
---

This is post number four in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Automata (in the formal languages / regex / parsing sense). It's also part two of the "implementation-heavy" stuff, where we go into implementing automata for real and useful things. This one is more of a mix of theory and code, which I hope is more appealing than the previous post which were either one or the other. In [part one]({% post_url 2016-10-03-implementing-finite-automata-part-1 %}) I naively claimed that this would be two posts of implementation, but I've since found more to write about. Plus it allows me to postpone the hardest part of this implementation stuff (benchmarking). 

This post will go into Infinite Sets, NFA-{%latex%}\varepsilon{%endlatex%} and the transitive reachability closures over cyclic graphs. 

# Infinite Sets

Something I ran into again at work recently is that regular languages can be finitely expressed with an automaton, even though they (usually) represent an infinite set of words. Every word in the language can be recognised by the finite automaton. So if our automata represent sets, we should be able to do set operations on them. 

So we already know how to test if a word is in the language, by applying our automaton on it. Something new would be union, complement, difference or intersection. Let's see how we can make sense of these

## Union

Say you have two automata that can recognise words in their respective languages. The union of the two would recognise words from either language. So creating one NFA for the combined language out of the two only requires that you run the two "in parallel" and if either ends up in a final state you accept. Let's try the naive way and merge the start states of the two automata:

{% digraph Binary string DFA 1 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q4 [shape=doublecircle, width=0.4];
start -> q0;
q0 -> q1 [label="1"];
q1 -> q2 [label="0"];
q2 -> q3 [label="0"];
q3 -> q3 [label="0"];
q3 -> q4 [label="1"];
q4 -> q4 [label="1"];
{% enddigraph %}

{% digraph Binary string DFA 2 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q9 [shape=doublecircle, width=0.4];
start -> q5;
q5 -> q6 [label="0"];
q6 -> q7 [label="1"];
q7 -> q8 [label="0"];
q8 -> q8 [label="0"];
q8 -> q9 [label="1"];
q9 -> q9 [label="1"];
{% enddigraph %}

{% digraph NFA union of DFAs 1 and 2 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q4 [shape=doublecircle, width=0.4];
start -> q0;
q0 -> q1 [label="1"];
q1 -> q2 [label="0"];
q2 -> q3 [label="0"];
q3 -> q3 [label="0"];
q3 -> q4 [label="1"];
q4 -> q4 [label="1"];
q9 [shape=doublecircle, width=0.4];
q0 -> q6 [label="0"];
q6 -> q7 [label="1"];
q7 -> q8 [label="0"];
q8 -> q8 [label="0"];
q8 -> q9 [label="1"];
q9 -> q9 [label="1"];
{% enddigraph %}

Cool. It works. But not in the general case! When there are edges going back to one of the start states, you get a mess. Say you have:

{% digraph Binary string DFA 3 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q2 [shape=doublecircle, width=0.4];
start -> q0;
q0 -> q1 [label="0"];
q1 -> q0 [label="0"];
q1 -> q2 [label="1"];
{% enddigraph %}

Odd number of zeroes, then a one, that's the language. And the other is:

{% digraph Binary string DFA 4 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q5 [shape=doublecircle, width=0.4];
start -> q3;
q3 -> q4 [label="1"];
q4 -> q5 [label="0"];
{% enddigraph %}

Single word language: {%latex%}\{10\}{%endlatex%}

Now we merge the start states naively as before:

{% digraph (wrong) NFA union of DFAs 3 and 4 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q2 [shape=doublecircle, width=0.4];
start -> q0;
q0 -> q1 [label="0"];
q1 -> q0 [label="0"];
q1 -> q2 [label="1"];
q5 [shape=doublecircle, width=0.4];
q0 -> q4 [label="1"];
q4 -> q5 [label="0"];
{% enddigraph %}

Now suddenly {%latex%}0010{%endlatex%} is in the combined language, but it wasn't in either of the originals. So that's not a proper union operation. To fix this, we'll use an extension of NFAs. 

### NFA-{%latex%}\varepsilon{%endlatex%}

A simple way to specify the correct union operation is through epsilon transitions. You add a new start state and epsilon transitions to the start states of the automata you want to union:

{% digraph (correct) NFA-e union of DFAs 3 and 4 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q2 [shape=doublecircle, width=0.4];
start -> q00;
q00 -> q0 [label="ε"];
q00 -> q3 [label="ε"];
q0 -> q1 [label="0"];
q1 -> q0 [label="0"];
q1 -> q2 [label="1"];
q5 [shape=doublecircle, width=0.4];
q3 -> q4 [label="1"];
q4 -> q5 [label="0"];
{% enddigraph %}

If you remember my [first finite automata post]({% post_url 2016-04-10-finite-automata %}#epsilon-moves) you may be reminded of the regex "or" (`|`) operator, which is exactly the same as this union of the underlying sets. The epsilon means you can take the transition "for free" without consuming any input. So it comes down to having a new start states that has the out-transitions of the all the old start states:

{% digraph (correct) NFA-e union of DFAs 3 and 4 %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q2 [shape=doublecircle, width=0.4];
q3 [color=grey, fontcolor=grey];
start -> q00;
q00 -> q1 [label="0"];
q00 -> q4 [label="1"];
q0 -> q1 [label="0"];
q1 -> q0 [label="0"];
q1 -> q2 [label="1"];
q5 [shape=doublecircle, width=0.4];
q3 -> q4 [label="1", color=grey, fontcolor=grey];
q4 -> q5 [label="0"];
{% enddigraph %}

This is a perfectly valid thing to do directly instead of going via epsilon transitions and doing using the epsilon-closure operation to turn the NFA-{%latex%}\varepsilon{%endlatex%} into a normal NFA. But at work a friend (Daniël) mentioned that I didn't go into epsilon closure in my last post, and that it's surprisingly hard to implement (efficiently anyway), so this time we'll look at this epsilon-closure. 

# The elusive epsilon closure

So we have an NFA-{%latex%}\varepsilon{%endlatex%} and we want a normal NFA. That means we need to copy the out-transitions of other nodes reachable by epsilon transition. Seems pretty simple right? Let's do that (naively), with some adapted data definitions from last post:

```rust
// This should look familiar :)
#[derive(Clone)]
struct NFAHashState<Input, StateRef, Payload> {
    transitions: HashMap<Input, HashSet<StateRef>>,
    payload: Option<Payload>,
}

#[derive(Clone)]
pub struct FiniteAutomaton<Input, State> {
    alphabet: Vec<Input>,
    states: Vec<State>,
}

type NFA<Input, Payload> = FiniteAutomaton<Input, NFAHashState<Input, usize, Payload>>;
type NFAE<Input, Payload> = FiniteAutomaton<Input, NFAHashState<Option<Input>, usize, Payload>>;
```

The `NFAE` is our augmented NFA, which can have a transition on `None` (the epsilon move) or on `Some(input)` (the normal transition). Although not quite accurate, we'll name our `NFAE -> NFA` operation `epsilon_closure`. I'm going to give you some **wrong** code that's simple for the first iteration:

```rust
impl<Input: Eq + Hash + Clone, Payload: Clone> NFAE<Input, Payload> {
    pub fn naive_epsilon_closure(&self) -> NFA<Input, Payload> {
        let mut epsilons: Vec<HashSet<usize>> = self.states
            .iter()
            .map(|st| st.transitions.get(&None).cloned().unwrap_or_else(HashSet::new))
            .collect();

        let mut states: Vec<NFAHashState<Input, usize, Payload>> = self.states
            .iter()
            .map(|st| {
                let transitions = st.transitions
                    .iter()
                    .filter_map(|(inp, st_ref)| inp.clone().map(|k| (k.clone(), st_ref.clone())))
                    .collect();
                NFAHashState::new(transitions, st.payload.clone())
            })
            .collect();

        for (n, eps) in epsilons.iter().enumerate() {
            for &e in eps {
                for (inp, st_ref) in states[e].transitions.clone() {
                    states[n].transitions.entry(inp).or_insert_with(HashSet::new).extend(st_ref);
                }
            }
        }

        NFA {
            alphabet: self.alphabet.clone(),
            states: states,
        }
    }
}
```

I know, collecting the `states` and `epsilons` can be done in a single loop. I chose this form for readability, and the compiler might do loop fusion[^loop-fusion]. Anyway, as you can see we just split up the normal transitions and epsilons. Then we copy over the normal transitions of a state the epsilon points to, to the origin of that epsilon. Easy peasy. Except, as I warned, it's **wrong**.

## Not as simple as that

Epsilon transitions don't consume any input. So you could as well take multiple before going over a "real" transition. This is what makes an epsilon closure a transitive closure. You can in principle reach more than one state away. That means that we could be doing extra work if we replace the epsilon transitions in an awkward order. 

You may be thinking of dynamic programming, caching or topological order to fix this problem. But there's another complication: cycles. Automata are just graphs, and cycles are allowed. An epsilon cycle basically means that the states in the cycle can be collapsed into one state. After all, they will all have the same out-transitions, so they'll have the same behaviour. 

So what I did is a Wikipedia search for a nice [transitive closure algorithm in graphs](https://en.wikipedia.org/wiki/Reachability). What I found when I did this was a [surprisingly simple, kind of disappointing algorithm](https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm#Algorithm). Here's the TL;DR: It's {%latex%}O(|V|^3){%endlatex%}, uses an adjacency matrix and extends that matrix in triply nested loops over the vertices. Not very exciting, so I tried to figure something out myself. I went through multiple iterations here as you [can](https://github.com/Apanatshka/dnfa/commit/730474af273a9f565e6f337561321a270a9c8b42) [see](https://github.com/Apanatshka/dnfa/commit/0d22e1c1ce80df4cff952bf6259dfc110fd84650), so wrong, some inelegant. I never got to the point of testing any of my implementations, because I'd discuss my progress at work with Daniël and I'd be motivated to find a nicer way to implement this thing. 

## The real implementation

So basically I was looking in the wrong direction while looking for an existing algorithm. My own implementation probably worked at some point but was quite inelegant, so I searched Wikipedia some different names. When you look at topological order instead of transitive closure, you'll find the issue of cycles. But if you follow the link on cycles, you get to [Strongly Connected Components](https://en.wikipedia.org/wiki/Strongly_connected_component). A strongly connected component (SCC) is a bunch of nodes that can reach all others in the SCC. So without epsilon cycles all your NFAE states are their own component in the epsilon-transition subgraph. Then when you look at the classic algorithms for getting SCCs in a graph, you'll find that [Tarjan's algorithm for SCCs](https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm) gives back these SCCs in reverse topological order. Exactly what we need!

When I found this I felt kind of silly. But I [implemented the Tarjan's SCC algorithm](https://github.com/Apanatshka/dnfa/commit/36bfaba5ded2a6784b811cb04056a1b50493f405), and it wasn't very hard. So let's walk through it. First, I moved the epsilon transition to a separate field in a new struct definition: 

```rust
// Much nicer to work with really. Why did I not do this before?
#[derive(Clone)]
struct NFAEHashState<Input, StateRef, Payload> {
    transitions: HashMap<Input, HashSet<StateRef>>,
    e_transition: HashSet<StateRef>,
    payload: Option<Payload>,
}
```

Then we have the start up part of the algorithm:

```rust
    /// This is an implementation of Tarjan's Strongly Connected Components algorithm. The nice
    /// property of this SCC algorithm is that it gives the SCC's in reverse topological order.
    fn scc(&self) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut index = 0;
        let mut st_index = vec![::std::usize::MAX; self.states.len()];
        let mut st_lowlink = vec![::std::usize::MAX; self.states.len()];
        let mut scc_stack = Vec::new();
        let mut stack_set = HashSet::new();
        let mut scc_s = Vec::new();

        for st_ref in 0..self.states.len() {
            if st_index[st_ref] == ::std::usize::MAX {
                self.scc_strongconnect(st_ref,
                                       &mut index,
                                       &mut st_index,
                                       &mut st_lowlink,
                                       &mut scc_stack,
                                       &mut stack_set,
                                       &mut scc_s);
            }
        }
        (st_lowlink, scc_s)
    }
```

In the end all this does is make sure we visit every node. We visit nodes with a depth-first search, that's what `scc_strongconnect` is for:

```rust
    fn scc_strongconnect(&self,
                         from: usize,
                         index: &mut usize,
                         st_index: &mut [usize],
                         st_lowlink: &mut [usize],
                         scc_stack: &mut Vec<usize>,
                         stack_set: &mut HashSet<usize>,
                         scc_s: &mut Vec<Vec<usize>>) {
        st_index[from] = *index;
        st_lowlink[from] = *index;
        *index += 1;

        scc_stack.push(from);
        stack_set.insert(from);

        for &to in &self.states[from].e_transition {
            if st_index[to] == ::std::usize::MAX {
                self.scc_strongconnect(to, index, st_index, st_lowlink, scc_stack, stack_set, scc_s);
                st_lowlink[from] = ::std::cmp::min(st_lowlink[from], st_lowlink[to]);
            } else if stack_set.contains(&to) {
                st_lowlink[from] = ::std::cmp::min(st_lowlink[from], st_index[to]);
            }
        }

        if st_lowlink[from] == st_index[from] {
            let mut scc = Vec::new();
            while let Some(st_ref) = scc_stack.pop() {
                stack_set.remove(&st_ref);
                scc.push(st_ref);
                if st_ref == from {
                    break;
                }
            }
            scc_s.push(scc);
        }
    }
```

So the `index` is a simple counter, and `st_index` and `st_lowlink` get a value from when you visit a node. But the lowlink should eventually hold the lowest index you can reach. So after you visit the children (or if you've already visited them) you propagate a lower lowlink back with the `min` function. Now the `scc_stack` is not only useful for tracking what the DFS has visited so far, but also to track SCCs. Any state that has no outgoing epsilon transitions will just keep `st_lowlink[from] == st_index[from]` and therefore be an SCC on its own. So it gets popped off of `scc_stack` and pushed onto `scc_s`, the reverse topo order SCCs vector. If a state did have epsilon transitions but it's children can't reach anything with a lower index than this one, then all states on the `scc_stack` from here on are one SCC with `st_index[from]` as an identifying number. So pop them off of `scc_stack` and add the lot to `scc_s`. 

If you can follow the code and my description of the algorithm, check out the [gif on wikipedia](https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm#/media/File:Tarjan%27s_Algorithm_Animation.gif). Visual explanations ftw. 

### [Using the SCCs](https://github.com/Apanatshka/dnfa/commit/c2463da9bc381a01fb1b09c9f989dcb90c56f50c)

```rust
    /// Replaces epsilon transitions with equivalent states/transitions
    /// Cycles are replaced by single states
    /// States that aren't reachable from `AUTO_START` are preserved (not by design)
    pub fn to_nfa(&self) -> NFA<Input, Payload> {
        // The SCCs are in reverse topo order for optimal epsilon
        let (sccs, renumbering) = self.scc();
        // the new states
        let mut states: Vec<NFAHashState<Input, usize, Payload>> = Vec::with_capacity(sccs.len());

        for scc in sccs {
            states[renumbering[scc[0]]] =
                Self::scc_to_nfa_state(&scc, &self.states, &renumbering, &states);
        }

        NFA {
            alphabet: self.alphabet.clone(),
            states: states,
        }
}
```

You may think: Is it really this tiny a function once you have the SCCs? But no, there is a helper function `scc_to_nfa_state` that's a bit larger. It *is* this **simple** though. We can get not only the SCCs out of Tarjan's algorithm in the desired reverse topo order, but we can also take the `st_lowlink` with. If we use it as a `renumbering` of the states, we'll effectively merge all the epsilon loops with no extra effort! Of course the `scc_to_nfa_state` does need to collect all the extra transitions, but you can do that from the new NFA that you're building because of the reverse topo order. So it's really just [30 lines of bookkeeping](https://github.com/Apanatshka/dnfa/blob/dfa16ae236fb21f5fdc6062c0496e18242f18c32/src/nfa.rs#L206-L236), being careful to use the renumbered state index iff necessary. 

## <del>Removing the recursion</del> <ins>Are we there yet?</ins>

Well.. Ok, let's stop here. I had something more to write about removing the recursion from the depth-first search, but I'll just link to it and not go into detail. My [first attempt](https://github.com/Apanatshka/dnfa/commit/d0ed9c2d2931f1fc62007c28bee8768abead3346) was a bit hairy, but I managed to [refactor it](https://github.com/Apanatshka/dnfa/commit/cc4b5ad939882a8fa18e50371dbe90ee23f70bf1) to something that I think is manageable. But it *is* more code, and not the clear algorithm is was anymore. So not as maintainable, and even though the algorithm will unlikely need any changes if it's bug-free[^bug-free], it may still need to be edited if the NFAE data structure changes. In the end I'd rather see the compiler do magic to eliminate recursive function call overhead. If I can do it, and it feels mechanical (it kind of did), then a compiler should be able to do this too right? But it's definitely non-trivial, so yeah.. Whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^loop-fusion]: I have no idea how much the compiler can currently do in this regard. Nor whether a single loop is even better in this case. 
[^bug-free]: I'm so sorry, but I didn't test any of the code in this blog post (or linked to in the post) *at all* O_O I'm a terrible person, I know. I thought of testing the code, but I was still exploring possible solutions. And I don't trust unit tests to really cover the whole problem and they can be brittle with this data layout for NFAs. Ideally I'd write property-based test ([quickcheck](https://crates.io/crates/quickcheck)), but that really requires extra code to normalise NFAs so structurally same NFAs have a unique representation. And I'd need to understand the problemspace fully to generate good input/output. So I thought of all this and I went "bleh, maybe later". In retrospect I should have just written a few unit tests...
