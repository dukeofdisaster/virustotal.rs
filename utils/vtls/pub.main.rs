/* Added loops for waiting
TODO:
  - add option for auto-submitting all urls
  - add option for file hashes

NOTE: 
  - structopt args are required unless eplicitly specified otherwise
  - SO FAR WILL BREAK IF IT GETS AN EMPTY LINE
  - Doesn't take into account the case where url's are submitted that 
    contain information that could be tied to a username; i.e /ahg=XDjdfjsdfk12321
    where somehow that's tied to the username... I fail to see how that's
    concerning, since there usually isn't a clear cut
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
// use std::{ thread, time, panic };

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
  let _api = "YOUR API KEY HERE";
  let domains = vec![ 
    "someinternaldomaintosanitize.com".to_string(),
    "someinternaldomaintosanitize.org".to_string(), 
    "anotherinternaldomain.net".to_string(),
    "otherdomain.com".to_string()];

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
  
  // - Can use a count if you're working on free tier api
 /* 
  if count > 4 {
    println!("URL length greater than 4; sleeping 60 seconds every 4 urls");
  }*/

  // we need a second count to see if we're at 4 urls, that way we can sleep
  //let mut count2 = 1;
  for i in lines {
    // We should check for empty lines and remove them. 
    // sanitize our input; trim as needed and replace any identifying info
    // like our domain. This variable needs to be mutable
    let mut tmpline = i.replace("\"", "");
    
    tmpline = tmpline.replace(" ", "");
    //println!("{}", tmpline);

    // We have to sleep here if we're on a free tier api
    // - comment out for prod release
    /*
    if count2 % 4 == 0 {
      //println!("Reached count: {} \n========", count2);
      thread::sleep(time::Duration::from_secs(3));
    }
    count2 += 1; */

    for j in &domains {
      if i.contains(j) {
        //println!("Found hit for internal domain ^^^^");
        tmpline = tmpline.replace(j, "TEST");
        println!("SANITIZED URL ==> {}", tmpline);
      }
    }

    // This might not be the best route from a performance
    // perspective because we check each username against the URL
    // however this guarantees that there are no username hits AND
    // the performance hit is neglible on the compiled binary
    for username in &users_vec {
      if tmpline.contains(username) {
        // redact the username
        tmpline = tmpline.replace(username, "XXX");
        //uncomment below for testing
        //println!("!!! USSERNAME REMOVED!!! => {}", tmpline);
      }
    } 

    // If the url doesn't have an http protocol identifier
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

    while scan.scan_id.is_none(){
      // We don't need to do anything here, we just need to wait for the value
      // to get initialized;
      continue;
    } 
    let id = &scan.scan_id.unwrap();
    //println!("ID: {}", id); 
    let mut fresh_report = url::report(api, id); //&scan.scan_id.unwrap());

    // Deals with waiting for the scan to be done
    while fresh_report.permalink.is_none() {
      fresh_report = url::report(api, id);
    }
    // Should never get here... 
    if fresh_report.permalink.is_none() {
      println!("{}", "NO RESULTS".bright_red());
      println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
      println!("{}", fresh_report.verbose_msg.bright_red());
      println!("========");
    }
    else{
      printresults(fresh_report);
      println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
      println!("========");
    }
    //println!("^^^^^ SCAN FAILOVER TEST ^^^^");
  }
  else {
    //println!("-- get_artifacts() else branch");
    // extracting the date string hear avoids the move error... 
    // this actually makes more sense... we should grab the string only
    // if we know it exists, thus we should only grab it after the variable res 
    // undergoes the validation in the if statement above
    let datestr = &res.scan_date.unwrap();
    if gooddate(&datestr) {
      let rep = url::report(api,url);
      printresults(rep);
      println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
      println!("========");
    } // 20190314: had to account for this which we never did
    else {
      println!("STALE SCAN... Submitting for rescan");

      // NOTE: needed to adjsut the scan date here
      // for some reason scan date is still the old date even though we
      // resubmitted it
      let scan = url::scan(api, url);
      while scan.scan_id.is_none(){
        continue;
      }
      let id = &scan.scan_id.unwrap();
      let mut fresh_report = url::report(api, id); //&scan.scan_id.unwrap());

      // Deals with waiting for the scan to be done
      // may try making this a continue loop as well... would need to check api stats..
      // ... may sometimes error out on high volumes of URLS
      while fresh_report.permalink.is_none() {
        fresh_report = url::report(api, id);
      }
      while !gooddate(&fresh_report.scan_date.unwrap()) {
        fresh_report = url::report(api,id);
      }
      let good_report = url::report(api,id); 
      // Should never get here... 
      if fresh_report.permalink.is_none() {
        println!("{}", "NO RESULTS".bright_red());
        println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
        println!("{}", fresh_report.verbose_msg.bright_red());
        println!("========");
      }
      else{
        printresults(good_report);
        println!("{} {}", "Scanned item => ".bright_green(), url.bright_green());
        println!("========");
      }
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

  // this is currently unused... can add in if you care about being within
  // a certain amount of days
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

