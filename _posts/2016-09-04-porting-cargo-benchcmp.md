---
layout:   post
title:    "Porting cargo benchcmp"
date:     2016-09-04
category: CompSci
tags:     [rust, cargo, tool, benchmark, benchcmp]
---

**TL;DR:** I've ported the tool [`cargo-benchcmp`](https://crates.io/crates/cargo-benchcmp) from [Python](https://github.com/BurntSushi/cargo-benchcmp/blob/1d23dec5dd3abe3939cfea030162a7dc6461e544/cargo-benchcmp) to [Rust](https://github.com/BurntSushi/cargo-benchcmp) and added some functionality. There is more to come which is mostly waiting for review in [pull requests](https://github.com/BurntSushi/cargo-benchcmp/pulls).

I've been messing around with Rust for a while now, and I found a little utility called [`cargo-benchcmp`](https://github.com/BurntSushi/cargo-benchcmp) by the famous [BurntSushi (Andrew Gallant)](http://blog.burntsushi.net/about/). You may have seen the benchmark comparisons in one of his [blogposts](http://blog.burntsushi.net/transducers/) already. I found out that the utility was a nice [single file Python script](https://github.com/BurntSushi/cargo-benchcmp/blob/1d23dec5dd3abe3939cfea030162a7dc6461e544/cargo-benchcmp). Not a quick and dirty hack, but classes and a few docstrings. I was already messing around with some of my own rust code where I wanted to do a comparison between benchmarks, so that's great. But the tool only worked with comparison of the same tests over time, and I wanted a comparison of the same tests on multiple implementations. So what do you do? Well, it's a small open source tool so I decided to contribute. That's what this post is about. 

# Porting the tool

I'm teaching myself Rust by writing in it, and my Python is a little rusty (heh), so I decided the first thing I'd do was port the tool to Rust. So I set out to find a good crate for handling command line arguments. And I found an implementation for [docopt](http://docopt.org/), which seems like a rather nice initiative. I can recommend the [Rust implementation of doctopt](https://github.com/docopt/docopt.rs) because it goes further than the basic API, and uses `rustc-serialize` and macros to get you a nice struct with all the command-line arguments with mostly the right types already. 

I also grabbed the [`regex`](https://crates.io/crates/regex) crate of course, to take the benchmark results apart. The regex for a line is the benchmark was something I could just copy, but I decided to pick it apart and [add comments](https://github.com/BurntSushi/cargo-benchcmp/blob/0512f17d1206f919706e1486e3dca4dd99068a39/src/benchmark.rs#L114-L119). The rest of the code started out in mostly the same structure as the Python code, with structs and functions instead of classes. 

Now for the output I wanted a nice table format and I found another crate for that which is very simple. It's called [`tabwriter`](https://crates.io/crates/tabwriter) and it's a port of the Go package for elastic tabstops. Which makes creating a table as simple as putting a `\t` (tab) character between you're columns. Given that that's what the character was originally intended for, it's a rather elegant solution. 

Grabbing all of these crates with `cargo` is very simple, and one of the pleasures of the Rust ecosystem. To make it even easier I did `cargo install cargo-edit`, which gives you three extra cargo subcommands: `list`, `add` and `rm`. So I could just `cargo add tabwriter` to get the latest version of the crate added to project dependencies. No more manual editing of `Cargo.toml`!

Another thing I learnt along the way was that cargo will just dispatch subcommands to whatever available `cargo-subcommand` executable on your `PATH`. So I didn't have to do anything special to make this Rust port available to Rust users. You can right now do `cargo install cargo-benchcmp`, and have `cargo benchcmp` just work. 

Something funny to note about the dependencies I've mentioned so far -- `docopt`, `regex` and `tabwriter` -- they're all written by BurntSushi. I couldn't have done this Rust port of his tool so easily without his other crates. 

# Comparing modules

In the project where I had benchmarks to compare, I generated different modules with the names of their implementation technique, all with the same benchmarks. This is fairly easy to [set up with a macro](https://github.com/Apanatshka/dnfa/blob/5cdb4307a06aee51f2d19f2619e0ff2e5e49af18/benches/basic.rs). The result is the same benchmark name, with different prefixes for the different implementations. So in the port of `benchcmp`, I added an option to provide two module names first, and the one or more files to read. The files would still contain the benchmark results, but the module names would be used to pick the benchmarks to compare. 

# The current help message

```
Compares Rust micro-benchmark results.

Usage:
    cargo benchcmp [options] <old> <new>
    cargo benchcmp [options] <old> <new> <file>
    cargo benchcmp -h | --help
    cargo benchcmp --version

The first version takes two files and compares the common benchmarks.

The second version takes two benchmark name prefixes and one benchmark output
file, and compares the common benchmarks (as determined by comparing the
benchmark names with their prefixes stripped). Benchmarks not matching either
prefix are ignored completely.

If benchmark output is sent on stdin, then the second version is used and the
third file parameter is not needed.

Options:
    -h, --help           Show this help message and exit.
    --version            Show the version.
    --threshold <n>      Show only comparisons with a percentage change greater
                         than this threshold.
    --variance           Show the variance of each benchmark.
    --improvements       Show only improvements.
    --regressions        Show only regressions.
```

# Things to come

I'm not done with this tool just yet. I did open a PR to the original repo and got a great code review from BurntSushi. He published it to [crates.io](https://crates.io/) too, and beat me to the punch by posting on Reddit, but I hope this post was still interesting to you. Here's some things I've been working on:

## Coloured output

Already during the first code review I figured out that my Rust port didn't get the output quite right. The table was already left-justified, whereas the original tool did right-justification of the last two columns. I liked the original output better so I switched `tabwriter` for [`prettytable-rs`](https://crates.io/crates/prettytable-rs), which is a nice ascii table generator. Out of the box it generated a lot of rules as well, but the format can be customised and one without any rules is even available as a preset `prettytable::format::consts::FORMAT_CLEAN`. The macros for the rows are well done, though I might have stumbled upon a regression where you *have* to give each cell the default format `d->"you cell here"` to get it to work. 

Anyway, that happened, I added right-justification. But the crate also gives the option to add colours to your table. So a simple change to the code, and now `benchcmp` will give you red rows for regressions and green rows for improvements ([Pull Request](https://github.com/BurntSushi/cargo-benchcmp/pull/9)). In my experience, this makes the options show only one or the other obsolete, but it doesn't hurt so it's still in there. 

## N-way comparison: Plotting

Warning: This feature may not actually make it into the tool proper. You can find [the discussion around that on github](https://github.com/BurntSushi/cargo-benchcmp/issues/8). For now this functionality lives in a [branch](https://github.com/Apanatshka/cargo-benchcmp/tree/plot). 

Comparing two implementations or commits at a time is great for day-to-day use and the table gives you detailed information. But when you just implemented a feature in a few different ways and found another crate that gives the same functionality, you really need an overview of how all of these things compare to each other. So I added a new subcommand `plot` to `benchcmp` (and moving the table functionality to subcommand `table`; I apologise in advance for the breaking change). 

The plot command compares by file or by module, any benchmark test that is present in multiple files/modules. It generated images in `target/benchcmp`, one file per benchmark, with a barchart/histogram plot + error bars for the variance. To generate the plots it uses `gnuplot`, which should be installed separately and put on your `PATH`. I suppose that's a weakness, but it was the easiest way to do this. Any suggestions to cut or simplify that dependency would be much appreciated, but I couldn't find a pure Rust plotting crate. If you're wondering, I don't use the [`gnuplot` crate](https://crates.io/crates/gnuplot), though I did try it at first. In the end it was easier to generate a gnuplot script myself than work with the limited subset of the crate, which does shallow bindings anyway. On the upside, I got to learn about gnuplot, which is a very handy (and powerful) tool indeed!

## Tests

One of the things that came up after the first release of the Rust version of `cargo-benchcmp` is tests. I kind of didn't write any... So I [asked](https://github.com/BurntSushi/cargo-benchcmp/issues/3) for a little advice on what to test, and got to work. So now I can report that `cargo-benchcmp` is pretty well-tested with [`quickcheck`](https://github.com/BurntSushi/quickcheck) for all the unit tests and [`second_law`](https://github.com/nathanross/second_law) for the integration tests ([Pull Request](https://github.com/BurntSushi/cargo-benchcmp/pull/10)). 

## Better statistics

There is [one last issue](https://github.com/BurntSushi/cargo-benchcmp/issues/4) on the repo for using better statistics when comparing benchmark measurements. I haven't looked into this one yet. We may need more information than `cargo bench` currently provides. Feel free you look into this one yourself dear reader. Just remember to outline your plan in the issue before you start doing major work. I made that mistake with the plotting feature...

## Showing comparisons with the tool

This post is a bit low on examples of `cargo benchcmp` output. I want to write a longer post on the implementation of finite automata with benchmark comparisons, along with performance traces and optimisations. But that may take some more time because I'm still figuring out how to use `perf` and `callgrind` etc., and my implementation is still squarely beaten by BurntSushi's [`aho-corasick`](https://github.com/burntsushi/aho-corasick) crate. But, you know, whatever ¯\\\_(ツ)\_/¯
