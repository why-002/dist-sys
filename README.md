# dist-sys
My current Implementation of the fly.io distributed systems challenges, written in rust.

# Challenges
This is currently the only project I have ever written in Rust, so this has been an interesting and informative way to start learning the new language.\
This also has been a great opportunity to take things I have learned in class like monads and expand it with other important programming concepts like mutexes, arcs, and closures.

# Test Echo

```bash

cd maelstrom

./maelstrom test -w echo --bin ~/dist-sys/target/debug/echo --node-count 1 --time-limit 10
```

# Test UUID Generation

```bash

cd maelstrom

./maelstrom test -w unique-ids --bin ~/dist-sys/target/debug/unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```
