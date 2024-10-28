# EFFS - Effect Filter Filesystem

A tool to view a filesystem, usually some part of it, filtered through
effects.

## Status

Currently, this is simply a proof of concept, the goal is to provide the
ability to mount a file (or files within some directory) and apply a
simple filter to them and present them at the mount point.

## Future plans

- Have a database per source for the filter required so the desired
  effects for the filtered views may be persisted.
- Support archive files, so whatever they contain may be accessed using
  standard utilities/tools that interact with files.
