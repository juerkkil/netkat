# netkat
A simple netcat clone written in Rust. Supports only IPv4 / TCP. Concurrency implemented using async model.

## Usage

```
# ./netkat --help
Usage: netkat [OPTIONS] [HOSTNAME] [PORT]

Arguments:
  [HOSTNAME]  Hostname (either destination address or the address to bind the listener)
  [PORT]      Port - either source or target port depending on mode of operation

Options:
  -l, --listen   Listen to incoming connection
  -v, --verbose  Verbose output
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

### Listen

```
./netkat -l 0.0.0.0 12345
```

### Connect

```
./netkat localhost 12345
```

### Sending and receiving files

Server hosting the local file `foo.gzip`

```
./netkat -l 0.0.0.0 12345 < foo.gzip
```

Client downloading the file and writing to local filesystem:

```
./netkat 1.2.3.4 12345 > foo.gzip
```

## Design principles

* Performance
* Simplicity
* Compatibility with netcat
* Writes only "business data" to stdout, and all other messages (info, error) to stderr.
* Not intended to replace netcat, but rather to provide an example for further network / async programming projects for rust.
