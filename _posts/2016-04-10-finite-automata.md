---
layout:   post
title:    "Finite Automata"
date:     2016-04-10
category: CompSci
tags:     [theory, basics, automata, computation]
---

Ever heard of [regular expressions](http://www.regular-expressions.info/) (regex)? I bet you have. They're a really handy programming tool for matching text, and even replacing it. But if you try to do too much with it at once (like, say, parsing a programming language) it gets really hairy, really quickly. All those lookaheads and lookbehinds and groups, smh. 

Did you know that regex in programming are usually more powerful than the original definition of the regular expression? Those powerful regular expressions are not really true to the word *regular* anymore. 

In this post I'll go into regular expressions and regular languages as they are defined in theoretical computer science. But to keep things interesting and understandable I'll give you runnable code snippets along the way. See [this announcement]({% post_url 2016-03-28-theory-of-computation %}) for more info on this series of posts, including a book I use for reference and the choice of programming language for the snippets.

# An exercise in minimalism

To see where regular expressions come from, let's look at a very minimal computer. We're going to look at *deterministic finite automata* (DFAs), which are simple state machines. 

## Finite

These DFAs take a single input string, which is finite. They have a finite number of states. They have a finite set of input symbols that they recognise. That's where the *finite* in DFA comes from, and it's all very explicitly mentioned so you can use these DFAs in proofs. *Whatever*, let's do something real with them instead! Let me just quickly finish the "what's with this long name" part. 

## Deterministic

Determinism means that the state machine know exactly what to do with every input in every state, and that there is only one possible thing to do. Ok, maybe that's too vague. Don't worry too much about it. It'll become clearer when we compare these DFAs with NFAs (*Non*-deterministic finite automata). 

## Examples or gtfo

Ok, let's make this concrete. We have a silly magic trap, a door which opens automatically when you walk towards it, but stays closed once you're on the other side. 

```
  FRONT          /---\  BACK        
  Opens          | D |  Trololol    
  Automatically  | o |  Now         
  On Detecting   | o |  You're      
  Someone Here   | r |  Stuck       
  It's Magic     \---/  I'm So Witty

```

The DFA state machine for this would like so:

{% digraph Magic Door DFA %}
rankdir=LR;
start [shape=none, label="", width=0];
node [shape=circle, fixedsize=shape, width=0.6];
start -> closed;
closed -> closed [label="BACK\nBOTH\nNEITHER"];
closed -> open [label="FRONT"];
open -> open [label="FRONT"];
open -> closed [label="BACK\nBOTH\nNEITHER", lp="200"];
{% enddigraph %}

So the start state is pointed at by an arrow coming from nowhere. The transitions between states are labeled with symbols (words in this case) from the input. The states are in the circles and are labeled with the state of the door. 

### Mathematical notation

A DFA definition consists of five parts, of which we have four in our example:

1. A set {% latex %}Q{% endlatex %} of *states*:
  {% latex %}\{\text{closed}, \text{open}\}{% endlatex %}
2. A set {% latex %}\Sigma{% endlatex %} of input symbols, this is called the *alphabet*:
  {% latex %}\{\text{FRONT}, \text{BACK}, \text{BOTH}, \text{NEITHER}\}{% endlatex %}
3. A function {% latex %}\delta{% endlatex %}, the *transition function*:
  This function is based on the arrows in the figure and the entries look like this: {% latex %}(\text{closed}, \text{FRONT}) \mapsto \text{open}{% endlatex %}. 
  Alternatively, you can put it in a table, with 'departure states' on the rows, inputs on the columns, and 'destination states' in the cells. 
4. A state {% latex %}q_0{% endlatex %}, the *start state*:
  {% latex %}\text{closed}{% endlatex %}
5. Crap, the example doesn't have this. Ok, ummm, we'll get to this part later in the post. 

### Get to the code already

Finally. Sorry. Let's do this in Rust:

```rust
{% include {{page.id}}/magic_door/src/main.rs %}
```

Wow, that was easier than I expected. The transition function turned out to be super simple. On input `Front` we go to state `Open`, on other inputs we go to state `Closed`. We don't even need to check what state we're in really. Kinda lame...

## Output

The wonky part of the magic door example is that we don't really have a finite input. Unless the door is destroyed, there is no obvious end, and therefore no definitive output. What you see in the code instead is that we observe the system and it's transitions. This is a traditional way to use a state machine in a large codebase as a kind of architectural pattern. But it's not what DFAs (or regular expressions for that matter) do. 

The simplest output is a simple "Yes" of "No". This is the fifth part of a DFA, and it works by marking some states as *accept states* ({% latex %}F{% endlatex %}, sometimes called *final states*). When the input ends, if the DFA is in an accept state, the output is "Yes" or "Accepted" or however you want to read it. In the other case the output is obviously "No". 

### Better example

Let's construct a DFA that can recognise inputs that start with a one, has at least two zeroes after that, and then at least one more one. 

{% digraph Binary string DFA %}
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

In Rust we can do the partial definition of the DFA with an `Option` type. 

```rust
{% include {{page.id}}/binary_string/src/main.rs %}
```

The transition function now returns an `Option<State>`, so `None` is the stuck state and the 'real' states are wrapped in a `Some`. The state in `main` is now wrapped in an `Option` too. And we just use `and_then` to apply the transition function to the state :)  
After the loop we now have a check to see if the state is in the final states. I wonder if it can be written in a shorter way... Either way, it's still pretty neat to use the library functions and handle these `Option`s fairly easily. 
