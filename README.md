# VirusTotal.rs

[![Build Status](https://travis-ci.org/owlinux1000/virustotal.rs.svg?branch=master)](https://travis-ci.org/owlinux1000/virustotal.rs)
[![MIT License](http://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE.txt)
[![Crates](https://img.shields.io/crates/v/virustotal.svg)](https://crates.io/crates/virustotal)

Library for VirusTotal API

## Implemented Features

| Resource                        | Description                        | Permission |
|:--------------------------------|:-----------------------------------|:-----------|
| GET /vtapi/v2/file/report       | Retrieve file scan reports         | public     |
| POST /vtapi/v2/file/scan        | Upload and scan a file             | public     |
| POST /vtapi/v2/file/rescan      | Rescanning already submitted files | public     |
| GET /vtapi/v2/url/report        | Retrieve URL scan reports          | public     |
| POST /vtapi/v2/url/scan         | Scan an URL                        | public     |
| POST /vtapi/v2/comments/put     | Make comments on files and URLs    | public     |
| GET /vtapi/v2/domain/report     | Retrieves a domain report          | public     |
| GET /vtapi/v2/ip-address/report | Retrieve an IP address report      | public     |

## Example

```
extern crate virustotal;

use virustotal::*;

fn main() {

    let api = "Your API KEY";
    let url = "The URL you want to check";
    let res = url::scan(api, url);
    println!("{:?}", url::report(api, &res.scan_id.unwrap()));
    
}
```

## Building one of the utilites
Just swap out the main Cargo.toml file with that of the utility you want and move pub.main.rs to src/ as main.rs then cargo run

## Acknowledgements

* [Thanks virustotal.com for posting](https://support.virustotal.com/hc/en-us/articles/115002146469-API-Scripts)
