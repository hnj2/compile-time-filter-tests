# Compile-time-filters tests

A collection of tests for the [`compile_time_filters`] feature for the [rust-lang/log] crate.

To test the [`compile_time_filtes`] feature yourself, you'll need to change the `Cargo.toml` of your application along these lines:
```diff
 ...

+[patch.crates-io]
+# patches log crate used in all dependencies
+log = { git = "https://github.com/hnj2/log.git", branch = "compile-time-filters" }        

 [dependencies]
 actix-web = "3"
 simple_logger = "1.5"
+# turns on the compile_time_features feature (also for dependencies)
+log = { version = "0.4", features = ["compile_time_filters"] }

 ...
```

## Tests

### Assembly
The [assembly](assembly/) crate is a basic example for how to use the filters, but also a demonstration that the the logging code can really be removed at compile time.

#### Usage

To get a baseline we'll just run the application with no filters:
```
$ cargo clean -p log
$ cargo run --bin assembly
...
     Running `target/debug/assembly`
2020-11-03 22:22:38,024 INFO  [assembly::a] A
2020-11-03 22:22:38,024 INFO  [assembly::a::a] AA
2020-11-03 22:22:38,024 INFO  [assembly::a::b] AB
2020-11-03 22:22:38,024 INFO  [assembly::b] B
```

To disable all but the log messages from `assembly::a::b` we can do the following:
```
$ cargo clean -p log
$ env RUST_LOG_FILTERS="Off; assembly::a::b=Trace" cargo run --bin assembly
...
     Running `target/debug/assembly`
2020-11-03 22:22:38,024 INFO  [assembly::a::b] AB
```

To disable all log messages but those from `assembly::a` except for those in `assembly::a::b` do:
```
$ cargo clean -p log
$ env RUST_LOG_FILTERS="Off; assembly::a=Trace; assembly::a::b=Off" cargo run --bin assembly
...
     Running `target/debug/assembly`
2020-11-03 22:22:38,024 INFO  [assembly::a] A
2020-11-03 22:22:38,024 INFO  [assembly::a::a] AA
```

#### Compile time? Zero cost?

All the logging calls are conveniently marked by enclosing `asm!("nop");` statements.
So using `objdump` we can examine the generated binary in the baseline case:
```
$ cargo clean -p log --release
$ cargo run --bin assembly --release
...
     Running `target/release/assembly`
2020-11-03 22:38:37,976 INFO  [assembly::a] A
2020-11-03 22:38:37,976 INFO  [assembly::a::a] AA
2020-11-03 22:38:37,976 INFO  [assembly::a::b] AB
2020-11-03 22:38:37,976 INFO  [assembly::b] B
...
$ objdump -Cd target/release/assembly
...

0000000000006b50 <assembly::main>:
    6b50:       53                      push   %rbx
    6b51:       48 83 ec 20             sub    $0x20,%rsp
    6b55:       48 89 e3                mov    %rsp,%rbx
    6b58:       48 89 df                mov    %rbx,%rdi
    6b5b:       ff 15 7f f0 24 00       callq  *0x24f07f(%rip)        # 255be0 <_GLOBAL_OFFSET_TABLE_+0x3c8>
    6b61:       48 89 df                mov    %rbx,%rdi
    6b64:       ff 15 56 ed 24 00       callq  *0x24ed56(%rip)        # 2558c0 <_GLOBAL_OFFSET_TABLE_+0xa8>
    6b6a:       84 c0                   test   %al,%al
    6b6c:       0f 85 b1 00 00 00       jne    6c23 <assembly::main+0xd3>

    6b72:       90                      nop
    6b73:       48 8d 1d ee f4 24 00    lea    0x24f4ee(%rip),%rbx        # 256068 <log::MAX_LOG_LEVEL_FILTER>
    6b7a:       48 8b 03                mov    (%rbx),%rax
    6b7d:       48 83 f8 03             cmp    $0x3,%rax
    6b81:       72 1e                   jb     6ba1 <assembly::main+0x51>
    6b83:       48 8d 3d d9 a7 03 00    lea    0x3a7d9(%rip),%rdi        # 41363 <anon.9f4ab4cfe4e3ea9393c0e815a93cf479.0.llvm.6163461553073396539>
    6b8a:       48 8d 0d 1f b5 24 00    lea    0x24b51f(%rip),%rcx        # 2520b0 <anon.9f4ab4cfe4e3ea9393c0e815a93cf479.3.llvm.6163461553073396539>
    6b91:       be 01 00 00 00          mov    $0x1,%esi
    6b96:       ba 03 00 00 00          mov    $0x3,%edx
    6b9b:       ff 15 4f ed 24 00       callq  *0x24ed4f(%rip)        # 2558f0 <_GLOBAL_OFFSET_TABLE_+0xd8>
    6ba1:       90                      nop

    6ba2:       90                      nop
    6ba3:       48 8b 03                mov    (%rbx),%rax
    6ba6:       48 83 f8 03             cmp    $0x3,%rax
    6baa:       72 1e                   jb     6bca <assembly::main+0x7a>
    6bac:       48 8d 3d d0 a7 03 00    lea    0x3a7d0(%rip),%rdi        # 41383 <anon.1484e048f2296be9ebcadced2968ba4c.0.llvm.11020714900902685602>
    6bb3:       48 8d 0d 2e b5 24 00    lea    0x24b52e(%rip),%rcx        # 2520e8 <anon.1484e048f2296be9ebcadced2968ba4c.3.llvm.11020714900902685602>
    6bba:       be 02 00 00 00          mov    $0x2,%esi
    6bbf:       ba 03 00 00 00          mov    $0x3,%edx
    6bc4:       ff 15 26 ed 24 00       callq  *0x24ed26(%rip)        # 2558f0 <_GLOBAL_OFFSET_TABLE_+0xd8>
    6bca:       90                      nop

...
```

If we now remove the log messages in the `assembly::a` module, there should be seven `nop` before the first logging code is executed:
```
$ cargo clean -p log --release
$ env RUST_LOG_FILTERS="assembly::a=Off" cargo run --bin assembly --release
...
     Running `target/release/assembly`
2020-11-03 22:38:37,976 INFO  [assembly::b] B
```
And voila:
```
$ objdump -Cd target/release/assembly
...

0000000000006a70 <assembly::main>:
    6a70:       53                      push   %rbx
    6a71:       48 83 ec 20             sub    $0x20,%rsp
    6a75:       48 89 e3                mov    %rsp,%rbx
    6a78:       48 89 df                mov    %rbx,%rdi
    6a7b:       ff 15 77 f1 24 00       callq  *0x24f177(%rip)        # 255bf8 <_GLOBAL_OFFSET_TABLE_+0x3c8>
    6a81:       48 89 df                mov    %rbx,%rdi
    6a84:       ff 15 4e ee 24 00       callq  *0x24ee4e(%rip)        # 2558d8 <_GLOBAL_OFFSET_TABLE_+0xa8>
    6a8a:       84 c0                   test   %al,%al
    6a8c:       75 3c                   jne    6aca <assembly::main+0x5a>
    6a8e:       90                      nop
    6a8f:       90                      nop
    6a90:       90                      nop
    6a91:       90                      nop
    6a92:       90                      nop
    6a93:       90                      nop
    6a94:       90                      nop
    6a95:       48 8d 05 cc f5 24 00    lea    0x24f5cc(%rip),%rax        # 256068 <log::MAX_LOG_LEVEL_FILTER>
    6a9c:       48 8b 00                mov    (%rax),%rax
    6a9f:       48 83 f8 03             cmp    $0x3,%rax
    6aa3:       72 1e                   jb     6ac3 <assembly::main+0x53>
    6aa5:       48 8d 3d 13 a7 03 00    lea    0x3a713(%rip),%rdi        # 411bf <anon.2b54de08100579310f96de1214351262.0.llvm.7694354358039187113>
    6aac:       48 8d 0d 4d b6 24 00    lea    0x24b64d(%rip),%rcx        # 252100 <anon.2b54de08100579310f96de1214351262.3.llvm.7694354358039187113>
    6ab3:       be 01 00 00 00          mov    $0x1,%esi
    6ab8:       ba 03 00 00 00          mov    $0x3,%edx
    6abd:       ff 15 45 ee 24 00       callq  *0x24ee45(%rip)        # 255908 <_GLOBAL_OFFSET_TABLE_+0xd8>
    6ac3:       90                      nop

...
```


### Actix
The [actix](actix/) crate is the hello world example for [actix-web].

Here we can test that this feature will also work for foreign crates.
With no filters the following messages will be logged:
```
$ cargo clean -p log
$ cargo run --bin actix
...
     Running `target/debug/actix`
2020-11-03 21:53:38,762 INFO  [actix_server::builder] Starting 8 workers
2020-11-03 21:53:38,762 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,762 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,762 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 INFO  [actix_server::builder] Starting "actix-web-service-127.0.0.1:8080" service on 127.0.0.1:8080
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,763 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,764 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,765 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,765 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,765 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,765 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,765 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,765 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,765 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,765 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,765 TRACE [mio::poll] registering with poller
2020-11-03 21:53:38,765 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,766 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 21:53:38,766 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
^C
2020-11-03 21:53:59,775 INFO  [actix_server::builder] SIGINT received, exiting
2020-11-03 21:53:59,775 TRACE [mio::poll] deregistering handle with poller
2020-11-03 21:53:59,776 TRACE [mio::poll] deregistering handle with poller
2020-11-03 21:53:59,776 TRACE [mio::poll] deregistering handle with poller
2020-11-03 21:53:59,777 TRACE [mio::poll] deregistering handle with poller
2020-11-03 21:53:59,777 TRACE [mio::poll] deregistering handle with poller
```
Let's say we want to raise the default log level to `Info`, but keep the `actix_server::worker` log levels at `Trace`:
```
$ cargo clean -p log
$ env RUST_LOG_FILTERS="Info; actix_server::worker=Trace" cargo run --bin actix
...
     Running `target/debug/actix
2020-11-03 22:03:45,304 INFO  [actix_server::builder] Starting 8 workers
2020-11-03 22:03:45,304 INFO  [actix_server::builder] Starting "actix-web-service-127.0.0.1:8080" service on 127.0.0.1:8080
2020-11-03 22:03:45,306 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,306 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,306 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,306 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,306 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,306 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,307 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
2020-11-03 22:03:45,307 TRACE [actix_server::worker] Service "actix-web-service-127.0.0.1:8080" is available
^C
2020-11-03 22:03:49,250 INFO  [actix_server::builder] SIGINT received, exiting
```

## Problems

Using clean and build every time you want to change the compile time configuration is annoying.
But at least it drives the point home, that this feature should only be used if it is really really necessary!

Notice the ugly `incomplete_featues` warning every time you compile this crate.
I know a way around using the `const_generics` feature.
It would be to generate the string prefix functions with the proc macro, but it is kind of ugly.

[actix-web]: https://github.com/actix/actix-web
[rust-lang/log]: https://github.com/rust-lang/log
[`compile_time_filters`]: https://github.com/hnj2/log/tree/compile-time-filters
