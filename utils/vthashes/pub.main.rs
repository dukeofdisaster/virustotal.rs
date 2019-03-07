/* building out SOC scripts with rust ftw

TODO:
  - add export to csv option ?
  - add output file option
*/
extern crate colored;
extern crate vthash;
extern crate structopt;
extern crate chrono;
extern crate serde_json;
use colored::*;

//use chrono::prelude::*;
use vthash::*;
//use std::fs::File;
//use std::io::{ BufRead, BufReader, Result}; 
use serde_json::json;
use std::fs;
//use std::{ thread, time, panic };
// for args; add more as needed
use structopt::StructOpt;


#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "-h", help = "a file of HASHES; agnosotic as to type")]
  hashfile: String,
  #[structopt(short="-j",required=false, help = "print out results as json-- only -j is needed", default_value="1")]
  json: u8,
}
fn main() {
  let _api = "xxxxxxYOUR API KEY HERExxxxxxxx";

  // Example usage
  // - Scan the filehash
  //let res = file::scan(api, url);
  // - get the report, given the scan_id of the scan we just submitted
  //println!("{:?}", file::report(api, &res.scan_id.unwrap()));
  let args = Cli::from_args();
  if args.json == 1 {
    println!("json: {}", args.json);
  }
  // import hashes from disk to a vector
  let hashes = fs::read_to_string(&args.hashfile).unwrap();
  let lines: Vec<_> = hashes.lines().collect();

  // We need to know the length of url vector since if it's > 4 we need to sleep
  // after submitting.... or give vt $$$..
  let count = lines.len();
  println!("# HASHES => {}", count);
  let mut unfound = 0;
  for i in lines {
    let res = file::report(_api, i);
    // Could just ignore all these and not print a shit load of messages
    if res.positives.is_none() {
      //println!("{} {}", "NO RESULTS:".bright_red(), i);
      //println!("res: {:?}", res);
      //println!("{} {}", "res.verbose_msg:".bright_red(), res.verbose_msg.bright_red());
      unfound = unfound +1;
    }
    else if args.json == 1 {
      let artifacts = json!({
        "permalink": res.permalink.unwrap(),
        "positives": res.positives.unwrap(),
        "sha1": res.sha1.unwrap(),
        "date": res.scan_date.unwrap()});
      println!("{}", artifacts.to_string().green());
    }
    else {
      println!("Permalink: {}", res.permalink.unwrap());
      println!("Positives: {}", res.positives.unwrap());
      println!("SHA1: {}", res.sha1.unwrap());
    }
  }
  println!("{} {}", "NUMBER OF HASHES WITH NO HITS IN VT: ".bright_red(), unfound.to_string().bright_red())
}    
