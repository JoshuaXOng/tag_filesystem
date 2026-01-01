# Tag Filesystem

A WIP FUSE filesystem based on the idea that sometimes it'd be useful if
directories didn’t have an ordering.

The filesystem utilizes files and tags. There are no traditional directories.
* Files can be marked with tags
* Files can be queried based on tags

For the most part, filesystem interactions re-use the typical CLI binaries (e.g., `rm`, `mv`, `cd`, etc.).

# Examples

```bash
# Installing the filesystem and setting-up helpers.
cargo install tag_filesystem
tfs tags setup

# Run in background, or run as a systemd service.
username@hostname:~/mnt$ nohup tfs iwanttags &
username@hostname:~/mnt$ cd iwanttags
username@hostname:~/mnt/iwanttags$ 

username@hostname:~/mnt/iwanttags$ mkdir tag_1 tag_2 tag_3
username@hostname:~/mnt/iwanttags$ touch file_1 file_2 file_3

username@hostname:~/mnt/iwanttags$ ls
tag_1
tag_2
tag_3

username@hostname:~/mnt/iwanttags$ ls "{}"
file_1
file_2
file_3

username@hostname:~/mnt/iwanttags$ mv file_1 "{ tag_1 }"
username@hostname:~/mnt/iwanttags$ mv file_2 "{ tag_1, tag_2 }"

username@hostname:~/mnt/iwanttags$ ls "{}"
file_3

username@hostname:~/mnt/iwanttags$ ls "{ tag_1 }"
file_1
tag_2

username@hostname:~/mnt/iwanttags$ ls "{ tag_1, tag_2 }"
file_2

username@hostname:~/mnt/iwanttags$ cd "{ tag_1 }"
username@hostname:~/mnt/iwanttags/{ tag_1 }$

username@hostname:~/mnt/iwanttags/{ tag_1 }$ mvf "{ ., tag_2 }"/file_2 .
username@hostname:~/mnt/iwanttags{ tag_1 }$ ls
file_1
file_2

# Contrived for the example (i.e., could have just done `cd {}`).
username@hostname:~/mnt/iwanttags{ tag_1 }$ ct ~tag_1
username@hostname:~/mnt/iwanttags$
```

# Contributing / Todo

Feel free to fork repo if you want.

Feel free to raise PRs to the repo.

Have scattered `TODO` comments around the codebase.

More broadly, want to eventually implement correct behaviour for core FUSE functions. And,
eventually get off of FUSE.

# Building

Pre-requisite, install the `capnp` binary, [steps here](https://capnproto.org/install.html).

Then, normal `cargo` commands from then on out.

# License

This project is licensed under the [MIT license](LISENSE).
