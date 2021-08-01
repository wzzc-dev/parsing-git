
读取 .git/objects/ 文件夹下 git对象

读取blob对象测试
```
cargo run --bin read_blob ../fixtures/blob_object/6e5f2c716e0071c51f6762090d1e4d7000826ea4
```

读取tree对象测试
```
cargo run --bin read_tree ../fixtures/tree_object/06ddd74dbc4f2877bf26f71ec8ccb2b094284e5c
```

读取commit对象测试
```
cargo run --bin read_blob ../fixtures/commit_object/e251d4845d8a538a22b1e2a6a4e804d01272efe9
```