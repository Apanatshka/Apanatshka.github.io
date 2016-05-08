---
layout:   post
title:    "Pushy Automata"
date:     2016-04-27
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

Welcome back! This is my second post in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Automata. I'll start with a quick refresher, but for more details read the [first post]({% post_url 2016-04-10-finite-automata %}). 

# Finite Automata refresher

Last time, we looked at deterministic and non-deterministic finite automata (DFAs and NFAs resp.), which can handle regular expressions (in fact, they are equivalent). They are finite state machines that only take input and return a binary accept/reject output. Finite automata *accept* when they end up in an accept state at the end of the input. You formally describe a finite automaton is by defining:

1. the allowable input symbols (or *alphabet* {%latex%}\Sigma{%endlatex%}), 
2. the *states* {%latex%}Q{%endlatex%}, 
3. the *state transitions* (as a finite mapping {%latex%}\delta{%endlatex%}), 
4. the *start state* {%latex%}q_0{%endlatex%} and
5. the *final* or *accept states* {%latex%}F{%endlatex%}.

DFAs require every state to have one and only one transition per input symbol. NFAs can have states that don't handle certain input symbols and states that have multiple transitions for the same input symbol. An NFA-{%latex%}\varepsilon{%endlatex%} even extends the alphabet with the empty string {%latex%}\varepsilon{%endlatex%}, so a transition doesn't have to consume input. The difference between NFAs and DFAs is not in expressive power (NFA execution is similar to translation to DFA + execution), but NFAs are easier to define. 

# Scotty, We Need More Power!

The non-regular language example from last post was: "Words in this language start with zeroes and after the zeroes are an equal number of ones". We can't count an arbitrary amount of zeroes in a finite number of states, so we can't remember how many ones we need to see at the end. Therefore, we need something extra: a *stack*. This stack makes our automaton equivalent in power to the context-free grammar. This type of grammar is used a lot in the reference manuals of programming languages and describes the lexer/parser setup. In fact, there are many tools that allow you to write a context-free grammar and generate a (lexer/)parser from it!  

## Pushdown Automata

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
The outer states are only there to make sure we have a fully empty stack before we go into a final state. The {%latex%}\${%endlatex%} is usually used as an *End Of Stack* or *End Of Input* character. You can also change the definition of the PDA to already hold 1 character on the stack at the start. This is part of the definition as you find it on [Wikipedia](https://en.wikipedia.org/wiki/Pushdown_automaton#Formal_definition). 

### Determinism

The previous PDA was non-deterministic, but we can to make it deterministic. Yes, I've left off the stuck state again. But otherwise, there should be no overlapping transitions and no transitions like {%latex%}\varepsilon, \varepsilon \rightarrow \_{%endlatex%}. 

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

If you think about it, it makes sense that a non-deterministic PDA is more powerful than a deterministic PDA. With the NFAs in the previous post we had just a finite amount of states we could be in while executing and you can model that with (an exponential amount of) states in a DFA. But for a PDA, it isn't just the state that matters, but the stack as well. Since the stack isn't finite, we can't just model it in more states. 

### Code

These PDAs are a bit annoying to write as is. But epsilons for the input character mean that we're not advancing the input, which feels wrong to me. We could do it and write a direct encoding of the formal definition, but then we need to resolve epsilons at runtime. The execution would become pull-based, asking for input and the top of the stack when we need it. Somehow that feels wrong to me. 
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
  - Fix?? Not sure if there is one, so we'll just have to deal with this one

With the above fixes, we can get a PDA that will always either advance one step in the input, or at the end of the input advance on the stack. I've been careful to only allow changes to our PDA definition that can be "easily expressed" in the original definition, so basically it still has the same power. Here's the new version of the PDA:

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

So with that, we can go to the code. I apologise for the messier transition functions. Those don't correspond to the diagram as clearly. In my defence: if we went for the simpler transition functions with explicit epsilons we would have basically needed an interpreter to actually execute the description. 

```rust
{% include {{page.id}}/binary_palindrome/src/main.rs %}
```

I left in two extra `println!` to observe the behaviour of the PDA:

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

Something to note is that the amount of `(state, stack)` tuples peaks at 3 and is mostly 2. That's not so bad. But in general you can have input strings with much worse behaviour!

## Context-free grammars

A context-free grammar (CFG) has consists of rules which are sometimes called production rules or substitution rules. Those names are basically two ways to look at the grammar: as a way to produce 'sentences' of the language that the grammar describes, or to reduce input to check if it's part of the language. 
These rules are written with terminals (symbols from the alphabet), and variables (or non-terminals). A variable is defined by one or more rules. Depending on the grammar formalism, you may see {%latex%}\leftarrow{%endlatex%}, {%latex%}\rightarrow{%endlatex%}, {%latex%}={%endlatex%} or {%latex%}::={%endlatex%} between the variable and the body of the rule. Let's look at an example:

:- | :-
{%latex%} S = 0 S 0 {%endlatex%}       | {%latex%} \text{(Rule-0)} {%endlatex%}
{%latex%} S = 1 S 1 {%endlatex%}       | {%latex%} \text{(Rule-1)} {%endlatex%}
{%latex%} S = \varepsilon {%endlatex%} | {%latex%} \text{(Rule-}\varepsilon\text{)} {%endlatex%}

This CFG describes the even-length binary palindromes that our last PDA also described. It has a single variable {%latex%}S{%endlatex%}, the *start variable* of the CFG. The zero and one are terminals, symbols from the alphabet. The epsilon is still the empty string. I've labelled the rules so I can refer to them later. 

To recognise a string, we start with the input and try to reduce part of it to a variable according to one of the rules. We keep substituting until we having only a single start variable left. For example:

:-: | :-
{%latex%} 0010101111010100 {%endlatex%} |
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

Of course the other way around also works, start with the start variable and expand variables cleverly to end up with the string to recognise. 

### Code

### Translation to PDA

Now CFGs equally powerful as the PDA. That means that like with regular expressions and DFAs, we can translate from one to the other. 

### Ambiguity

### Pumping lemma
