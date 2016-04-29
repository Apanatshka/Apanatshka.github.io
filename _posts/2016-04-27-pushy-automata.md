---
layout:   post
title:    "Pushy Automata"
date:     2016-04-10
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

Welcome back! This is my second post in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Automata. I'll start with a quick refresher, but for more details read the [first post]({% post_url 2016-04-10-finite-automata %}). 

# Finite Automata refresher

Last time, we looked at deterministic (DFA) and non-deterministic finite automata (NFA), which can handle regular expressions (in fact, they are equivalent). They are finite state machines that only take input and return a binary accept/reject output. Finite automata *accept* when they end up in an accept state at the end of the input. You formally describe a finite automaton is by defining:

1. the allowable input symbols (or *alphabet* {%latex%}\Sigma{%endlatex%}), 
2. the *states* {%latex%}Q{%endlatex%}, 
3. the *state transitions* (as a finite mapping {%latex%}\delta{%endlatex%}), 
4. the *start state* {%latex%}q_0{%endlatex%} and
5. the *final* or *accept states* {%latex%}F{%endlatex%}.

DFAs require every state to have one and only one transition per symbol. NFAs can have states that don't handle certain input symbols and states that have multiple transitions for the same input symbol. An NFA-{%latex%}\varepsilon{%endlatex%} even extends the alphabet with the empty string {%latex%}\varepsilon{%endlatex%}, so a transition doesn't have to consume input. The difference between NFAs and DFAs is not in expressive power (NFA execution is similar to translation to DFA + execution), but NFAs are easier to define. 

# We need more power

You may remember the non-regular language example "Words in this language start with zeroes and after the zeroes are an equal number of ones". We can't count an arbitrary amount of zeroes in a finite number of states, so we can't remember how many ones we need to see at the end. Therefore, we need something extra: a *stack*. This stack makes our automaton equivalent in power to the context-free grammar. This type of grammar is used a lot in the reference manuals of programming languages and describes the lexer/parser setup. In fact, there are many tools that allow you to write a context-free grammar and generate a (lexer/)parser from it!  

## Pushdown Automata

The finite automaton that has a stack is called a pushdown automaton (PDA). You can think of the stack as a tray dispenser that you can *push* new trays *down* on. Let's kick things off with an example PDA that recognises our non-regular language example:

{% digraph Non-regular language example %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q0 [shape=doublecircle, width=0.4];
q3 [shape=doublecircle, width=0.4];
start -> q0;
q0 -> q1 [label="ε, ε → $"];
q1 -> q2 [label="1, 0 → ε"];
q2 -> q3 [label="ε, $ → ε"];
q1 -> q1 [label="0, ε → 0"];
q2 -> q2 [label="1, 0 → ε"];
{% enddigraph %}


