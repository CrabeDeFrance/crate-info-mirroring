# crate-info-mirroring

Application used to download metadata from a crate directory.
This directory and subdirectories should be in crates.io format : 1  2  3  a-  A-  a0  a1  a2 ...

You can get a sample of this directory format by cloning crates.io git directory or using panamx to download packages. 

Output directory will follow input directory structure format.
Both directories must exist.

Uses same crate than cargo show to fetch metadata from crates.io. 

To install:

```sh
$ cargo install --path . 
```

Usage:

```sh
$ crate-info-mirroring --help
```

And:

```sh
$ crate-info-mirroring -i <input_crate_directory> -o <output_crate_metadata_directory>
```

It is possible to use a configuration file. In this case, command line options will override configuration in file (except for verbosity, the system will take the most verbose configuration).

By default, this application will look for a file named 'crate-info-mirroring.toml' in the current directory. It is possible to change path to this file by using -c or --config option.

Example of config file:

```toml
# input directory
input = "./path/to/input_crate_directory"
# output directory
output = "./path/to/output_crate_metadata_directory"
# number of concurrent processes
count = 16
# log level filtering ( Off, Error, Warn, Info, Debug, Trace )
verbose = "Error"
```
