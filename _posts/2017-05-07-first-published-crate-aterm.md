---
layout:   post
title:    "My first published crate: aterm"
date:     2017-05-07
category: CompSci
tags:     [annotated term, term, Rust]
---

I published my first crate to [crates.io](https://crates.io/)! It's called [`aterm`](https://crates.io/crates/aterm), and it's a library that implements the Annotated Term format. Currently it can only parse and print the normal textual format, but I'm planning to add the other three formats too at some point. There are also a number of other improvements that I have planned. But I'm going to try to not make this post a braindump of meandering thoughts. So here's what I'd like to talk about today:

* Table of Contents
{:toc}

# What are ATerms

The <u>A</u>nnotated <u>Term</u> format[^1] originates from the [Centre for Mathematics and Computer Science (CWI)](https://www.cwi.nl/about) in Amsterdam. It describes trees and annotations on those trees. The big features are maximal sharing (subtrees that are the same are only allocated once), a compressed binary format, and a C implementation that does garbage collection. The format dates back to around 2000. 

The ATerm format was used by a large set of tools in a number of research groups as an exchange format (and they probably also used the library implementations to provide the internal representation as well). Maximal sharing was super fancy, although recently research was published that this maximal sharing can in many situations be more overhead than optimisation. [^2]

For those of you who are interested in implementations of this format in other languages, have a looks at the [CWI repository](https://github.com/cwi-swat/aterms) for ATerm implementations, which includes implementations in C, Java and C#. 

# Why implement this

As mentioned in the high-level description of the ATerm format, this was used by a large number of tools created by researchers as an exchange format. I actually use this format myself in my work as a PhD candidate. So I figured I could combine my wish to learn more Rust and performance engineering, with some of the knowledge I have about aging tools I use at work! 

In particular, I'm currently working on an interpreter, written in Rust, that interprets Stratego Core code. [Stratego](http://strategoxt.org/) (or Stratego/XT to disambiguate from the boardgame) is a language for program transformation. The compiler can return an intermediate representation called the "core" language. This is an AST, which it prints in the ATerm format. I will write about this interpreter in another blog post at a later time. 

# The implementation

# Things I learned

# What happened to the automata?

Eh, whatever ¯\\\_(ツ)\_/¯

[^1]: van den Brand, M. G., De Jong, H. A., Klint, P., & Olivier, P. A. (2000). Efficient annotated terms. *Software Practice and Experience*, 30(3), 259-291.

[^2]: Steindorfer, M. J., & Vinju, J. J. (2016, March). Performance Modeling of Maximal Sharing. In *Proceedings* of the 7th ACM/SPEC on *International Conference on Performance Engineering* (pp. 135-146). ACM.