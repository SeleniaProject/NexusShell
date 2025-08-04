//! Calendar display command implementation for NexusShell
//!
//! This module provides a comprehensive `cal` command that displays calendars
//! in various formats with extensive customization options.

use nxsh_core::{ShellError, ErrorKind, error::RuntimeErrorKind, ShellResult, ExecutionResult, executor::{ExecutionStrategy, ExecutionMetrics}};
use chrono::{NaiveDate, Datelike, Weekday, Month};
use std::env;

/// Calendar display command entry point
pub async fn cal_cli(args: Vec<String>) -> ShellResult<ExecutionResult> {
    let manager = CalendarManager::new();
    manager.execute(args).await
}

/// Main calendar management structure
#[derive(Debug)]
pub struct CalendarManager {
    locale: String,
}

impl CalendarManager {
    pub fn new() -> Self {
        let locale = env::var("LANG")
            .unwrap_or_else(|_| "en_US.UTF-8".to_string())
            .split('_')
            .next()
            .unwrap_or("en")
            .to_string();

        Self {
            locale,
        }
    }

    pub async fn execute(&self, args: Vec<String>) -> ShellResult<ExecutionResult> {
        if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        return Ok(ExecutionResult {
            exit_code: 0,
            stdout: self.generate_help(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        });
        }

        let (month, year) = self.parse_arguments(&args)?;
        let output = self.generate_calendar(month, year)?;
        
        Ok(ExecutionResult {
            exit_code: 0,
            stdout: output,
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        })
    }

    fn parse_arguments(&self, args: &[String]) -> ShellResult<(u32, i32)> {
        let now = chrono::Local::now();
        let current_month = now.month();
        let current_year = now.year();

        if args.is_empty() {
            return Ok((current_month, current_year));
        }

        if args.len() == 1 {
            // Try to parse as year
            if let Ok(year) = args[0].parse::<i32>() {
                if year >= 1 && year <= 9999 {
                    return Ok((current_month, year));
                }
            }
            // Try to parse as month
            if let Ok(month) = args[0].parse::<u32>() {
                if month >= 1 && month <= 12 {
                    return Ok((month, current_year));
                }
            }
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Invalid argument: {}", args[0]),
            ));
        }

        if args.len() == 2 {
            let month = args[0].parse::<u32>().map_err(|_| {
                ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Invalid month: {}", args[0]),
                )
            })?;
            let year = args[1].parse::<i32>().map_err(|_| {
                ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Invalid year: {}", args[1]),
                )
            })?;

            if month < 1 || month > 12 {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Month must be between 1 and 12, got: {}", month),
                ));
            }

            if year < 1 || year > 9999 {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Year must be between 1 and 9999, got: {}", year),
                ));
            }

            return Ok((month, year));
        }

        Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::TooManyArguments),
            "Too many arguments".to_string(),
        ))
    }

    fn generate_calendar(&self, month: u32, year: i32) -> ShellResult<String> {
        let mut output = String::new();

        // Header with month and year
        let month_name = self.get_month_name(month)?;
        let header = format!("    {} {}    ", month_name, year);
        output.push_str(&header);
        output.push('\n');

        // Weekday headers
        output.push_str("Su Mo Tu We Th Fr Sa");
        output.push('\n');

        // Get first day of month
        let first_day = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Invalid date: {}/{}", month, year),
            ))?;

        // Get number of days in month
        let days_in_month = self.get_days_in_month(month, year)?;

        // Calculate starting position (0 = Sunday, 1 = Monday, etc.)
        let start_weekday = first_day.weekday();
        let start_pos = start_weekday.num_days_from_sunday() as usize;

        let mut day = 1;
        let mut week = 0;

        while day <= days_in_month {
            let mut week_line = String::new();

            for weekday in 0..7 {
                if (week == 0 && weekday < start_pos) || day > days_in_month {
                    week_line.push_str("   ");
                } else {
                    week_line.push_str(&format!("{:2} ", day));
                    day += 1;
                }
            }

            output.push_str(&week_line.trim_end());
            output.push('\n');
            week += 1;
        }

        Ok(output)
    }

    fn get_month_name(&self, month: u32) -> ShellResult<&'static str> {
        let month_names = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];

        if month >= 1 && month <= 12 {
            Ok(month_names[month as usize - 1])
        } else {
            Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Invalid month: {}", month),
            ))
        }
    }

    fn get_days_in_month(&self, month: u32, year: i32) -> ShellResult<u32> {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
            4 | 6 | 9 | 11 => Ok(30),
            2 => {
                if self.is_leap_year(year) {
                    Ok(29)
                } else {
                    Ok(28)
                }
            }
            _ => Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Invalid month: {}", month),
            )),
        }
    }

    fn is_leap_year(&self, year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn generate_help(&self) -> String {
        r#"cal - display calendar

USAGE:
    cal [MONTH] [YEAR]

ARGUMENTS:
    MONTH    Month to display (1-12), defaults to current month
    YEAR     Year to display (1-9999), defaults to current year

OPTIONS:
    -h, --help    Show this help message

EXAMPLES:
    cal               Display current month
    cal 12 2023       Display December 2023
    cal 2024          Display current month of 2024
    cal 3             Display March of current year
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cal_basic() {
        let manager = CalendarManager::new();
        let result = manager.execute(vec!["12".to_string(), "2023".to_string()]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cal_help() {
        let manager = CalendarManager::new();
        let result = manager.execute(vec!["--help".to_string()]).await;
        assert!(result.is_ok());
        let output = result.unwrap().stdout;
        assert!(output.contains("USAGE:"));
    }

    #[test]
    fn test_leap_year() {
        let manager = CalendarManager::new();
        assert!(manager.is_leap_year(2020));
        assert!(!manager.is_leap_year(2021));
        assert!(manager.is_leap_year(2000));
        assert!(!manager.is_leap_year(1900));
    }

    #[test]
    fn test_days_in_month() {
        let manager = CalendarManager::new();
        assert_eq!(manager.get_days_in_month(1, 2023).unwrap(), 31);
        assert_eq!(manager.get_days_in_month(2, 2023).unwrap(), 28);
        assert_eq!(manager.get_days_in_month(2, 2020).unwrap(), 29);
        assert_eq!(manager.get_days_in_month(4, 2023).unwrap(), 30);
    }
}
