/* building out SOC scripts with rust ftw
TODO:
  - add option for auto-submitting all urls (to have fresh date)
  - add option for json output like vthashes util
NOTE: 
  - structopt args are required unless eplicitly specified otherwise
  - this can be easily modified by keeping a count of how many api calls have been made
  - then sleeping to stay within bounds if you're testing out the free tier
*/

extern crate colored;
extern crate vtls;
extern crate structopt;
extern crate chrono;
use colored::*;
use chrono::prelude::*;
use vtls::*;
use std::fs;
// Below used to force sleeping to make free api happy
//use std::{ thread, time, panic };

// for args; add more as needed
use structopt::StructOpt;
#[derive(StructOpt)]
struct Cli {
  #[structopt(help = "a file of URLS -- missing protocol identifier will be added")]
  urlfilename: String,
  #[structopt(help = "a file of NAMES (i.e. all users in your environment) to check for and remove from urls")]
  usersfile: String,
}

fn main() {
  let _api = "xxxxINSERT YOUR API KEY HERExxxxx"
  let domains = vec!["HERE YOU'D INSERT A LIST OF YOUR INTERNAL DOMAINS".to_string(), 
    "domain2".to_string(),
    "domain3".to_string()];

  // Get our args
  let args = Cli::from_args();

  // read urlfile to a vector
  let urls = fs::read_to_string(&args.urlfilename).unwrap();
  let lines: Vec<_> = urls.lines().collect();

  // read usersfile to a vector
  let users = fs::read_to_string(&args.usersfile).unwrap();
  let users_vec: Vec<_> = users.lines().collect();


  // We need to know the length of url vector since if it's > 4 we need to sleep
  // after submitting.... or give vt $$$
  let count = lines.len();
  println!("COUNT => {}", count);
  
  // - comment out for prod release
 /* 
  if count > 4 {
    println!("URL length greater than 4; sleeping 60 seconds every 4 urls");
  }*/

  // we need a second count to see if we're at 4 urls, that way we can sleep
  //let mut count2 = 1;
  for i in lines {
    
    // sanitize our input; trim as needed and replace any identifying info
    // like our domain. This variable needs to be mutable
    let mut tmpline = i.replace("\"", "");
    
    //println!("{}", tmpline);

    // We have to sleep here if we're on a free tier api
    // - comment out for prod release
    /*
    if count2 % 4 == 0 {
      //println!("Reached count: {} \n========", count2);
      thread::sleep(time::Duration::from_secs(3));
    }
    count2 += 1; */

    // here we borrow domains; & == borrow
    for j in &domains {
      if i.contains(j) {
        // uncomment below for testing
        //println!("Found hit for internal domain ^^^^");
        tmpline = tmpline.replace(j, "TEST");
        // uncomment below for testing
        println!("SANITIZED URL ==> {}", tmpline);
      }
    }

    // This might not be the best route from a performance
    // perspective because we check each username against the URL
    // however this guarantees that there are no username hits AND
    // the performance hit is neglible on the compiled binary
    // (at least with our number of users: ~3.1K)
    for username in &users_vec {
      if tmpline.contains(username) {
        // redact the username
        tmpline = tmpline.replace(username, "XXX");
        //uncomment below for testing
        //println!("!!! USSERNAME REMOVED!!! => {}", tmpline);
      }
    } 

    // If the username doesn't have an http protocol identifier
    // insert it
    if !i.contains("http") {
      //println!("url doesn't have http://");

      // Here we need a way to append the protocol identifier
      // otherwise VT throw's a fit
      let mut protocol: String = "http://".to_owned();

      // concatenates the protocol identifier with the string missing it
      protocol.push_str(&tmpline);
      tmpline = protocol;
      //println!("PROTOCOL ADDITION TEST ++++++ > {}", tmpline); 
      //panic!();
    }
    // Finally... get all the artifacts
    get_artifacts(_api, &tmpline);
  } 
}

// ================================== FUNCTION DEFINITIONS =============================================
//
//
//
// takes an API and a URL, and retrieves the artifacvts fro ma report
// NOTE: the UrlReporotResponse type does not impelement copy so we should be break it down
// to the actual values compared if we can.. i.e string, u32, etc
fn get_artifacts(api: &str, url: &str) {
  //println!("get_artifacts() TEST: {} , {}", api, url);

  // NEED: error checking here
  let res = url::report(api, url);
  // If we get nones here we should pivot, submit a URL for scan, shit that
  // result out, then continue to skip to the next item
  if (res.positives == None) || (res.permalink == None) || (res.scan_date == None) {
    println!("{}", "NONE RESULT... submitting for scan".bright_red());
    let scan = url::scan(api, url);
    //println!("scan initialized");
    //println!("Scan id: {:?}", &scan.scan_id.unwrap());

    // NEED: Error checking here; this often returns an Err, in which case we should probably
    //       loop until we get an Ok
    //
    //sleep here too allow the scan to process;
    // increased to 15 seconds because scan timing is inconsistent
    // NOTE: no date validation is needed herer since a scan submission
    // will always return a present date
    //thread::sleep(time::Duration::from_secs(45));
    //println!("Thread done sleeping");
    let id = &scan.scan_id.unwrap();
    //println!("ID: {}", id); 
    let fresh_report = url::report(api, id); //&scan.scan_id.unwrap());
    printresults(fresh_report);
    println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
    println!("========");
    //println!("^^^^^ SCAN FAILOVER TEST ^^^^");
  }
  else {
    //println!("-- get_artifacts() else branch");
    // extracting the date string hear avoids the move error... 
    // this actually makes more sense... we should grab the string only
    // if we know it exists, thus we should only grab it after the variable res 
    // undergoes the validation in the if statement above... DUH!
    let datestr = &res.scan_date.unwrap();
    if gooddate(&datestr) {
      let rep = url::report(api,url);
      printresults(rep);
      println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
      println!("========");
    }
  }
}

// takes in a UrlReportResponse and extracts what we care about
// from it... mainly the link / how many positives / and when it was scanned
fn printresults(resp: UrlReportResponse) {
  let response = resp;
  //println!("PRINTRESULTS FUNCTION TEST\n###########");
  let link = &response.permalink.unwrap();
  let positives = &response.positives.unwrap();
  let date = &response.scan_date.unwrap();
  println!("{} {}\n{} {}\n{} {}", 
    "Permalink => ".bright_green(), link.bright_green(),
    "Positives => ".bright_green(), positives.to_string().bright_green(), 
    "Scan Date => ".bright_green(), date.bright_green());
}

fn gooddate(date: &str) -> bool {
  //let date = resp.scan_date.unwrap();
  let yearstr = date[..4].to_string();
  let monthstr = date[5..7].to_string();
  //let daystr = date[8..10].to_string();

  // get the numbers; year has to be i32 because chrono is inconsistent and ghey
  let yearnum = yearstr.parse::<i32>().expect("Failed to parse yearstr");
  let monthnum = monthstr.parse::<u32>().expect("Failed to parse monthstr");

  // this is currently unused... will debate with team whether or not
  // we care about day as long as it was in current month
  //let daynum = daystr.parse::<u32>().expect("Failed to parse dasytr");
  //println!("Goddate yearnum: {}\ncurrentyear(): {}", yearnum, currentyear());
  if yearnum == currentyear() && monthnum == currentmonth() {
    //println!("Yearnum EQ currentyear and monthnum EQ currentmonth");
    return true;
  }
  return false;
}
/*
fn currentday() -> u32  {
  let dt = Local::now();
  return dt.day();
}*/

fn currentmonth() -> u32 {
  let dt = Local::now();
  return dt.month();
}

// Why TF is this an i32 when alll the others are u32 ??
fn currentyear() -> i32 {
  let dt = Local::now();
  return dt.year();
}

