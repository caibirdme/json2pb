[![master Actions Status](https://github.com/caibirdme/json2pb/workflows/master/badge.svg)](https://github.com/caibirdme/json2pb/actions)

### json2pb
json2pb is a simple cli tool that can convert a json object to a protobuf message.

This is really useful when you refactor your obsolete restful APIs.

Converting a json request and response to pb message, and add your own service definition, cool!

### install
* you must install `rust` dev environment first.
* `git clone https://github.com/caibirdme/json2pb.git`
* `cd {dir}/json2pb`
* `cargo install --path . --bin j2pb`

### usage
j2pb -h

```
j2pb 0.1.0
ronaldoliu@tencent.com
convert json to protobuf3

USAGE:
    j2pb [OPTIONS] -f <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <file>        specify json file name
    -o <out>         specify file to save generated proto(default: stdout)

```

example:
```
j2pb -f test.json
# this will print generated protobuf message on the screen(stdout)
```

```
j2pb -f test.json -o test.proto
# this will create test.proto file and save generated protobuf message in it
```

### NOTICE
protobuf itself doesn't support nested array, so you can't convert json with nested array, like:
```json
{
  "foo": [
    [1,2,3],
    [4,5,6]
  ]
}
```

And json itself just supports double type, but most often, people use it to store integer.
So json2pb will try to convert a double value T to int64 if it satisfies both the two constraints:
* (T.floor()-T).abs() < f64::EPSILON // means T is an integer
* `-(2^53-1) <= T <= 2^53-1`  // double can only store integer in this range precisely
