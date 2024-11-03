# netkat
A simple netcat clone written in Rust. Supports IPv4/IPv6 and TCP/UDP. Concurrency implemented using async model.

## Usage

```
# ./netkat --help
Usage: netkat [OPTIONS] [HOSTNAME] [PORT]

Arguments:
  [HOSTNAME]  Hostname (either destination address or the address to bind the listener)
  [PORT]      Port - either source or target port depending on mode of operation

Options:
  -l, --listen             Listen to incoming connection
  -u, --udp                Use UDP instead of TCP
  -t, --timeout <TIMEOUT>  Timeout in seconds
  -v, --verbose            Verbose output
  -h, --help               Print help
  -V, --version            Print version
```

## Examples

### Listen

IPv4:

```
./netkat -l 0.0.0.0 12345
```

IPv6:

```
./netkat -l :: 12345
```

You can also bind to a specific IP address if you wish to only listen e.g. localhost or some particular network interface.

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
