[![master Actions Status](https://github.com/caibirdme/json2pb/workflows/master/badge.svg)](https://github.com/caibirdme/json2pb/actions)
[![Crates.io Version](https://img.shields.io/crates/v/json2pb.svg)](https://crates.io/crates/json2pb)

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

#### proto doesn't support nested array
protobuf itself doesn't support nested array, so you **can't** convert json with nested array, like:
```json
{
  "foo": [
    [1,2,3],
    [4,5,6]
  ]
}
```

#### convert double to int64 if possible

JSON itself just supports double type, but most often, people use it to store integer.
So json2pb will try to convert a double value T to int64 if it satisfies both the two constraints:
* (T.floor()-T).abs() < f64::EPSILON // means T is an integer
* `-(2^53-1) <= T <= 2^53-1`  // double can only store integer in this range precisely

#### Automatically choose the most possible object as the message def
We always see JSON like this:
```json
{
  "bar": [
    {
      "name": "deen"
    },
    {
      "name": "caibirdme",
      "age": 26
    }
  ]
}
```
Due to some reasons, some element may be not complete, so json2pb will choose the most possible object(object with most keys) as the message definition.

So the generated message will be:
```
message root_data {
    repeated Bar bar = 1;
    
    message Bar {
        string name = 1;
        int64 age = 2;
    }   
}
```

#### Empty array
In json, there's always empty array, like:
```json
{
  "foo": []
}
```
This case, json2pb cannot inference its type. So the only way is to convert it as:
```proto
import "google/protobuf/any.proto";

message root_data {
    repeated google.protobuf.Any foo = 1;
}
```
And make sure your target language support Any type.