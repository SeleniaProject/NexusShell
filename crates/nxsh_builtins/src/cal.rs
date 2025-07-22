//! `cal` builtin – display a calendar.
//!
//! Supported modes:
//!   cal             # current month
//!   cal YEAR        # full year calendar (YEAR e.g. 2025)
//!   cal MONTH YEAR  # specific month (1-12) and year
//!
//! Option `-y` behaves like `cal YEAR`.
//! Week starts on Monday to follow POSIX cal default (can be adjusted later).

use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate};
use std::fmt::Write as _;

pub async fn cal_cli(args: &[String]) -> Result<()> {
    let mut month: Option<u32> = None;
    let mut year: Option<i32> = None;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-y" => {
                idx += 1;
                if idx >= args.len() {
                    // -y alone => show current year
                    let now = Local::now();
                    year = Some(now.year());
                } else {
                    year = Some(parse_year(&args[idx])?);
                    idx += 1;
                }
            }
            arg => {
                if year.is_none() {
                    // first numeric treated as year unless month not set
                    if month.is_none() {
                        // decide by remaining args count
                        if idx + 1 < args.len() {
                            // treat as month
                            month = Some(parse_month(arg)?);
                            idx += 1;
                            year = Some(parse_year(&args[idx])?);
                        } else {
                            year = Some(parse_year(arg)?);
                        }
                    } else {
                        year = Some(parse_year(arg)?);
                    }
                } else {
                    return Err(anyhow!("cal: too many arguments"));
                }
            }
        }
        idx += 1;
    }

    if year.is_none() && month.is_none() {
        // current month
        let now = Local::now();
        month = Some(now.month());
        year = Some(now.year());
        print_month(month.unwrap(), year.unwrap());
    } else if month.is_none() {
        // yearly calendar
        let yr = year.unwrap();
        for m in 1..=12 {
            print_month(m, yr);
            if m != 12 {
                println!();
            }
        }
    } else {
        print_month(month.unwrap(), year.unwrap());
    }

    Ok(())
}

fn parse_month(s: &str) -> Result<u32> {
    let m: u32 = s.parse()?;
    if m == 0 || m > 12 {
        return Err(anyhow!("cal: invalid month"));
    }
    Ok(m)
}

fn parse_year(s: &str) -> Result<i32> {
    let y: i32 = s.parse()?;
    Ok(y)
}

fn print_month(month: u32, year: i32) {
    let header = month_name(month);
    println!("     {} {}", header, year);
    println!("Mo Tu We Th Fr Sa Su");
    // determine first weekday (Monday=1..Sunday=7)
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut weekday = first.weekday().number_from_monday();
    let days_in_month = days_in_month(year, month);

    let mut day = 1;
    // leading spaces
    for _ in 1..weekday {
        print!("   ");
    }
    while day <= days_in_month {
        print!("{:>2} ", day);
        if weekday == 7 {
            println!();
            weekday = 1;
        } else {
            weekday += 1;
        }
        day += 1;
    }
    if weekday != 1 {
        println!();
    }
}

fn month_name(m: u32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let last_day_next_prev = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap().pred_opt().unwrap();
    last_day_next_prev.day()
} 