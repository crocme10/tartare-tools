# Contributing to `tartare-tools`

We welcomes contribution from everyone in the form of suggestions, bug
reports, pull requests, and feedback. This document gives some guidance if you
are thinking of helping us.

## Submitting bug reports and feature requests

When reporting a bug or asking for help, please include enough details so that
the people helping you can reproduce the behavior you are seeing. For some tips
on how to approach this, read about how to produce a [Minimal, Complete, and
Verifiable example].

[Minimal, Complete, and Verifiable example]: https://stackoverflow.com/help/mcve

When making a feature request, please make it clear what problem you intend to
solve with the feature, any ideas for how `transit_model` could support solving
that problem, any possible alternatives, and any disadvantages.

### Internal work management tool

At Kisio Digital (ex. CanalTP) we track tasks and bugs using a private tool.
This tool is private but we sometimes refer to it when submitting
PRs (those `Ref. ND-123`), to help later work.
Feel free to ask for more details if the description is too narrow,
we should be able to provide information from tracking tool if there is more.

## Checking quality

We encourage you to check that the formatting, static analysis and test suite
passes locally before submitting a pull request with your changes. If anything
does not pass, typically it will be easier to iterate and fix it locally than
waiting for the CI servers to run tests for you.

### Formatting

We use the standard Rust formatting tool, [`rustfmt`].

```sh
# To format the source code in the entire repository
make format
```

[`rustfmt`]: https://github.com/rust-lang/rustfmt

### Static analysis

For the static analysis, we use [`clippy`].

```sh
# Check lints on the source code in the entire repository
make lint
```

[`clippy`]: https://github.com/rust-lang/rust-clippy

### Tests

The test suite include unit test and integration tests.

```sh
# Run all the tests of `tartare-tools` in the entire repository,
make test
```

## Environments and tools

At Kisio Digital, we mostly maintain, test and operate on the following
environments and tools:

* Our main target for OS is [Debian].
* Our main target for [PROJ] is the version described in the
  [main README](README.md#PROJ-for-binaries).

However, we are open to contributions to help support more of them.

[Debian]: https://www.debian.org
[PROJ]: https://proj.org

## Working with `transit_model`

`tartare-tools` is very dependent on `transit_model`. Here are some information
to work with it.

### Upgrading `transit_model`

When a new major version of `transit_model` is released, you need to bump the
version in every crate of the repository (the main `Cargo.toml` and also in all
the folders of the workspace).

If only a new minor version of `transit_model` is released, you don't need to do
anything else than `cargo update` on your local `tartare-tools`.

### Testing a non-released version of `transit_model`

Sometimes, modifications have been applied on `transit_model` but not yet
released. You might need to check these modifications won't break
`tartare-tools`, or you might want to prepare a future PR with these new
features of `transit_model`.

To do that, you have 2 options.

⚠In both cases, these are only temporary measures, they should not be merged
into the `master` branch.⚠

#### Local `transit_model`

If you checked out the version of `transit_model` you want to use on your
machine, then you can add in `tartare-tools/Cargo.toml` the following section.

```toml
[patch.crates-io]
transit_model = { version = "x.y.z", features = ["proj"], path = "/path/to/transit_model" }
```

#### Remote branch on `transit_model`

If the functionality of `transit_model` is already pushed on a branch on Github,
you can do the following.

```toml
[patch.crates-io]
transit_model = { version = "x.y.z", features = ["proj"], git = "https://github.com/<user>/transit_model.git", branch = "<feature-branch>" }
```

Then don't forget to do:

```sh
cargo update
```

The advantage of this second solution is that you will be able to open a PR to
have first feedbacks because the CI will be able to build your branch.

## Conduct

We follow the [Rust Code of Conduct].

[Rust Code of Conduct]: https://www.rust-lang.org/conduct.html
