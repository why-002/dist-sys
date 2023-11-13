# dist-sys
My current Implementation of the fly.io distributed systems challenges (https://fly.io/dist-sys/), written in rust.

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

./maelstrom test -w unique-ids --bin ~/dist-sys/target/debug/unique-id --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```

# Test Broadcast
3a - Single Node
```bash

cd maelstrom

./maelstrom test -w broadcast --bin ~/dist-sys/target/debug/broadcast --node-count 1 --time-limit 20 --rate 100
```

3b - Multi-node with gossip
```bash

cd maelstrom

./maelstrom test -w broadcast --bin ~/dist-sys/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
```

3c - Multi-node with partition
```bash

cd maelstrom

./maelstrom test -w broadcast --bin ~/dist-sys/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition
```

3d - Efficent Broadcast Part 1
```bash

cd maelstrom

./maelstrom test -w broadcast --bin ~/dist-sys/target/debug/broadcast_d --node-count 25 --time-limit 20 --rate 100 --latency 100
```

3e - Efficent Broadcast Part 2
```bash

cd maelstrom

./maelstrom test -w broadcast --bin ~/dist-sys/target/debug/broadcast_e --node-count 25 --time-limit 20 --rate 100 --latency 100
```

# Test Counter
```bash

cd maelstrom

./maelstrom test -w g-counter --bin ~/dist-sys/target/debug/counter --node-count 3 --rate 100 --time-limit 20 --nemesis partition
```

# To-do
<ul>
  <li>Impl a send function for struct Message</li> 
</ul>
