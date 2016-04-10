---
layout:   post
title:    "Finite Automata"
date:     2016-04-10
category: CompSci
tags:     [theory, basics, automata, computation]
---

What do Turing machines and regular expressions have in common? One is a theoretical model of a computer, and can be used to prove that some things cannot be computed. The other is a practical tool for matching strings. And yet they are both based on a simpler 'computational model': a (very constrained) finite state machine (FSM). 

In this blog post we'll go over the basics of this type of FSM and instead of going over proofs, we'll go over examples and little implementations in Rust. For more information about this blog post series, see [this announcement post]({% post_url 2016-03-28-theory-of-computation %}). 

# An exercise in minimalism

We'll start with the simplest, most restricted version of our FSMs. These are great for proofs because everything explicitly defined and super simple. But they are not so great to construct by hand, so we'll discuss a nicer version afterwards and relate that to regular expressions. 

## Deterministic Finite Automaton (DFA)

DFAs are FSMs (*automata*) that work on a *finite* input and give a boolean output. *True* means the input was recognised as part of the 'language' that the DFA encodes, *false* means it is not part of the language. *Deterministic* automata define all their (*finite* amount of) states and *exactly one* transition for every possible pair of state and input. 
The way you formally describe a DFA is by defining:

1. the *states*, 
2. allowable input symbols (or *alphabet*), 
3. the *state transitions* (as a finite function), 
4. the *start state* and
5. the *final* or *accept states*.

### Example: Binary string

Let's construct a DFA that can recognise inputs that start with a one, has at least two zeroes after that, and then at least one more one, after which the 'word' ends. 

{% digraph Binary string DFA %}
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

Note here that I've already started cheating with the construction of this DFA. Not every state handles all symbols in the alphabet (0 and 1). This partially defined DFA is usually easier to write and read. The usual way to make it fully defined is to add an explicit *stuck state*, which is the error state (you could call it a sink). All the unhandled symbols go to that state, and with any input the DFA will stay in that state. At the end of the input the output is then that the input was not recognised. If you program this you might be able to fail early depending on how you use your DFA. 

### Code code code

In Rust we can do the partial definition of the DFA with an `Option` type:

```rust
{% include {{page.id}}/binary_string/src/main.rs %}
```

So `None` is the stuck state and the 'real' states are wrapped in a `Some`. 

### Memory

Note that DFAs are so restricted that they don't really have mutable memory. Any kind of memory of what you've already seen of the input needs to be statically encoded in the states of the state machine. This can get a little awkward when you want to recognise binary strings that have a 1 as the second to last symbol:

{% digraph Binary string DFA %}
layout="circo";
bgcolor="transparent";
rankdir=LR;
start [shape=none, label="", width=0];
node [shape=doublecircle, fixedsize=shape, width=0.4, mindist=2];
q100 [label="100"];
q101 [label="101"];
q110 [label="110"];
q111 [label="111"];
node [shape=circle, fixedsize=shape, width=0.5];
q000 [label="000"];
q001 [label="001"];
q010 [label="010"];
q011 [label="011"];
start -> q000;
q000 -> q000 [label="0"];
q000 -> q001 [label="1"];
q001 -> q010 [label="0"];
q001 -> q011 [label="1"];
q010 -> q100 [label="0"];
q010 -> q101 [label="1"];
q011 -> q110 [label="0"];
q011 -> q111 [label="1"];
q100 -> q000 [label="0"];
q100 -> q001 [label="1"];
q101 -> q010 [label="0"];
q101 -> q011 [label="1"];
q110 -> q100 [label="0"];
q110 -> q101 [label="1"];
q111 -> q110 [label="0"];
q111 -> q111 [label="1"];
{% enddigraph %}

We remember the last three input symbols in our states, which gives us {%latex%}2^3{%endlatex%} states. An exponential relation, that's not good for practical use, nor for designing these DFAs. 

## Nondeterministic Finite Automaton (NFA)

Nondeterminism allow states to have however many transitions per symbol that it wants. Basically that means that when you simulate such an NFA, you can be in multiple states at once. This allows us to avoid the exponential blowup of the last example:

{% digraph Binary string DFA %}
bgcolor="transparent";
rankdir=LR;
start [shape=none, label="", width=0];
node [shape=doublecircle, fixedsize=shape, width=0.4, mindist=2];
q4 [label="1.."];
node [shape=circle, fixedsize=shape, width=0.5];
q1 [label="..."];
q2 [label="..1"];
q3 [label=".1."];
start -> q1;
q1 -> q1 [label="0,1"];
q1 -> q2 [label="1"];
q2 -> q3 [label="0,1"];
q3 -> q4 [label="0,1"];
{% enddigraph %}

Although this NFA is easier to describe, it's still always translatable to a DFA. This translation algorithm is called powerset construction or subset construction. The powerset of a set is the set of all combinations: {%latex%}\mathbb{P}(\{0,1\}) = \{\emptyset, \{0\}, \{1\}, \{0,1\}\}{%endlatex%} 

The powerset algorithm uses the powerset of the states of the NFA to create the states for the DFA. That's where the exponential blowup comes from, **if** all those states are used. The state with the {%latex%}\emptyset{%endlatex%} is the *stuck state* that we already saw earlier. The start state is still the same although now called {%latex%}\{q_o\}{%endlatex%} instead of {%latex%}q_o{%endlatex%}. The final states are every state that has an NFA final state in its set. 

* Operators
  * Concatenate
  * Options (union)
  * Kleene star
* Limitations and Nonregular languages
  * Pumping lemma
  * Variable memory, though limited. Left for next blog post
