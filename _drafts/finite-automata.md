---
layout:   post
title:    "Finite Automata"
date:     2016-03-28
category: CompSci
tags:     [theory, basics, automata, computation]
---

Ever heard of [regular expressions](http://www.regular-expressions.info/) (regex)? I bet you have. They're a really handy programming tool for matching text, and even replacing it. But if you try to do too much with it at once (like, say, parsing a programming language) it gets really hairy, really quickly. All those lookaheads and lookbehinds and groups, smh. 

Did you know that regex in programming are usually more powerful than the original definition of the regular expression? Those powerful regular expressions are not really true to the word *regular* anymore. 

In this post I'll go into regular expressions and regular languages as they are defined in theoretical computer science. But to keep things interesting and understandable I'll give you runnable code snippets along the way. See [this announcement]({% post_url 2016-03-28-theory-of-computation %}) for more info on this series of posts, including a book I use for reference and the choice of programming language for the snippets.

# An exercise in minimalism

To see where regular expressions come from, let's look at a very minimal computer. We're going to look at *deterministic finite automata* (DFAs), which are simple state machines. 

## Finite

These DFAs take a single input string, which is finite. They have a finite number of states. They have a finite set of input symbols that they recognise. That's where the *finite* in DFA comes from, and it's all very explicitly mentioned so you can use these DFAs in proofs. *Boring*, let's do something *real* with them instead. I'll just quickly finish the "what's with this long name" part of the text. 

## Deterministic

Determinism means that the state machine know exactly what with every input in every state, and that there is only one possible thing to do. Ok, maybe that's too vague. Let's look at an example.

## Example

We have a silly magic trap, a door which opens automatically when you walk towards it, but stays closed once you're on the other side. 

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
node [shape=circle, width=0.8];
start [shape=none, label="", width=0];
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
  I'm not going to write all of this, it's the arrows in the figure and looks like a lot of these: {% latex %}(\text{closed}, \text{FRONT}) \mapsto \text{open}{% endlatex %}. Alternatively, can also put it in a table, with 'departure states' on the rows, inputs on the columns, and 'destination states' in the cells. 
4. A state {% latex %}q_0{% endlatex %}, the *start state*:
  {% latex %}\text{closed}{% endlatex %}
5. We'll get to this part later. 

### Get to the code already

Finally. Sorry. Let's do this in Rust:

```rust
{% include 2016-03-28-finite-automata-magic-door.rs %}
```

Wow, that was easier than I expected. The transition function turned out to be super simple. On input `Front` we go to state `Open`, on other inputs we go to state `Closed`. We don't even need to check what state we're in really. 

## Output


