# cold-stat

![Crates.io Version](https://img.shields.io/crates/v/cold-stat)

A CLI tool to statically analyze the initialization time, known as cold-start, of AWS Lambda functions.

<img src="https://raw.githubusercontent.com/exoego/cold-stat/main/doc/img.png" width="855" height="104" />

## Installation

```bash
cargo install cold-stat
```

## Usage

```
cold-stat [OPTIONS] --function <FUNCTION> --payload <PAYLOAD>
```

### Options:
-  `-f`, `--function <FUNCTION>`
    - Name or ARN of function to invoke
-  `-p`, `--payload <PAYLOAD>`
    - JSON payload to send to the function
-  `--log-group-name <LOG_GROUP_NAME>`
    - Name of CloudWatch log group to analyze
    - [default: `/aws/lambda/FUNCTION`]
-  `--log-stream-filter <LOG_STREAM_FILTER>`
    - Regex to filter CloudWatch log group stream. Useful when log group is shared by multiple functions
    - [example: `/YOUR-FUNCTION-NAME/`] when log streams are named like `2021/01/01/YOUR-FUNCTION-NAME[$LATEST]`
-  `-i`, `--iterations <ITERATIONS>`
    - Number of iterations to invoke the function
    - It is recommended to set `30` at least. Because the number of collected cold starts often is a bit shorter than the specified `ITERATIONS` due to eventual consistency of CloudWatch Logs
    - [default: `100`]
-  `-v`, `--verbose`
    - Print debug logs if enabled
-  `-h`, `--help`
    - Print help
-  `-V`, `--version`
    - Print version

### Result
<img src="https://raw.githubusercontent.com/exoego/cold-stat/main/doc/img.png" width="855" height="104" />

- `mem`
  - Memory size of the function 
- `count`
  - Number of cold starts collected 
- `stddev`, `min`, `max`
  - Standard deviation, minimum, and maximum of cold start time respectively
- `p50`, `p90`, `p95`, `p99`, `p995`, `p999`
  - Percentiles of cold start time
  - For example, `p50` is 50 percentile, also known as the median
  - `p995` and `p999` are 99.5 and 99.9 percentiles, respectively

## Development

### Build

```bash
git clone https://github.com/exoego/cold-stat
cd cold-stat
cargo build
```

### Run

```bash
cargo run -- \
  --function=YOUR-FUNC-NAME \
  --iterations=10 \
  --verbose \
  --payload='{"foo": "bar"}'
```

## Acknowledgement

- This tool is highly inspired by [lumigo-io/SAR-measure-cold-start](https://github.com/lumigo-io/SAR-measure-cold-start)
