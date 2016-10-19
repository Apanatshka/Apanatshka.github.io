---
layout:   post
title:    "Implementing Finite Automata (Part 2)"
date:     2016-10-16
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

This is a perfectly valid thing to do directly instead of going via epsilon transitions and doing using the epsilon-closure operation to turn the NFA-{%latex%}\varepsilon{%endlatex%} into a normal NFA. But a friend mentioned that I didn't go into epsilon closure in my last post, and that it's surprisingly hard to implement, so this time we'll look at this epsilon-closure. 

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

I know, collecting the `states` and `epsilons` can be done in a single loop. I chose this form for readability. Anyway, as you can see we just split up the normal transitions and epsilons. Then we copy over the normal transitions of a state the epsilon points to, to the origin of that epsilon. Easy peasy. Except, as I warned, it's **wrong**.

## Not as simple as that

Epsilon transitions don't consume any input. So you could as well take multiple before going over a "real" transition. This is what makes an epsilon closure a transitive closure. You can in principle reach more than one state away. That means that we could be doing extra work if we replace the epsilon transitions in an awkward order. 

You may be thinking of dynamic programming, caching or topological order to fix this problem. But there's another complication: cycles. Automata are just graphs, and cycles are allowed. An epsilon cycle basically means that the states in the cycle can be collapsed into one state. After all, they will all have the same out-transitions, so they'll have the same behaviour. 

So let's cheat and do a Wikipedia search for a nice [transitive closure algorithm in graphs](https://en.wikipedia.org/wiki/Reachability). What I found when I did this was a [surprisingly simple, kind of disappointing algorithm](https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm#Algorithm). Here's the TL;DR: It's {%latex%}O(|V|^3){%endlatex%}, uses an adjacency matrix and extends that matrix in triply nested loops over the vertices. I'm sure there's better stuff out there, but instead of diving into papers and deciphering their particular flavour of pseudo-code, I spent an afternoon making something up. 

## The real implementation

Soo.. I went through multiple iterations here as you [can](https://github.com/Apanatshka/dnfa/commit/730474af273a9f565e6f337561321a270a9c8b42) [see](https://github.com/Apanatshka/dnfa/commit/0d22e1c1ce80df4cff952bf6259dfc110fd84650) in my git history. But in between I discussed my implementation with the same friend who pointed out that this was a hard exercise. And he suggested a better way. But let's start at the beginning, the rationale:

A transitive closure algorithm is nice and dandy, but we're not sure getting the epsilon-reachable set for one state, we'll be going over all of them. We want to avoid doing double work by copying the transitions of all epsilon-reachable states, when we can just copy only the transitions of directly epsilon-reachable states[^1] *after* we've already copied over transition in those states. That is to say: we want to go over these epsilons in reverse-[topological order](https://en.wikipedia.org/wiki/Topological_sorting); or without using jargon: (1) States without outgoing epsilon-transitions already have all their normal transitions -- they're already done. (2) States that have epsilon-transitions to only those "done" states can just copy only those transitions over and then become done. (3) States that have epsilon-transitions to states that also have epsilon-transitions should wait until those states they have epsilon-transitions to become "done", then we only need to copy their transitions. 

Of course topological order doesn't actually work when you have cycles. But in our case we know how we want to interpret cycles: as a single state. So if we do the right combination of topo-sort + cycle detection, we can efficiently build our NFA without epsilons. 

To the code then! What we'll do is a depth-first search[^2], which should be linear time[^3] if we ignore the costs copying the transitions. That's a stupid way of saying we're probably still in the quadratic or cubic ballpark like the Warshall algorithm I linked to earlier, but whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^1]: reachable in one epsilon-transition
[^2]: it's cool that it's that simple, and it's one of the algorithms mentioned on the Wikipedia. Also, my friend suggested this too, thanks Daniël!
[^3]: even cooler: this problem is in complexity class [NC](https://en.wikipedia.org/wiki/NC_(complexity)) which means you can get polylogarithmic time complexity when you on a parallel computer with a polynomial number of processors. 
