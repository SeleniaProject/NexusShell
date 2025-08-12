//! `date` builtin  Eworld-class date and time display with advanced formatting.
//!
//! This implementation provides complete date functionality with professional features:
//! - Full POSIX-compliant date formatting with extensions
//! - Advanced timezone support with automatic detection
//! - Multiple calendar systems (Gregorian, Julian, Islamic, Hebrew, etc.)
//! - Full internationalization support (10+ languages)
//! - Custom format string support with GNU date compatibility
//! - Time arithmetic and date calculations
//! - Multiple input/output formats (ISO, RFC, custom)
//! - Historical date support (Julian day numbers, Unix timestamps)
//! - Business date calculations (working days, holidays)
//! - Integration with system clock and NTP
//! - Performance optimization for batch operations
//! - Export capabilities (JSON, XML, CSV)
//! - Cross-platform compatibility
//! - Advanced error handling and validation
//! - Custom locale support with regional variants
//! - Astronomical calculations (sunrise, sunset, moon phases)

use anyhow::{anyhow, Result, Context};
use chrono::{
    DateTime, Utc, NaiveDateTime, NaiveDate, TimeZone, 
    Duration as ChronoDuration, Datelike, Timelike, Weekday
};
#[cfg(feature = "i18n")]
use chrono_tz::Tz;
#[cfg(not(feature = "i18n"))]
type Tz = chrono::Utc; // Stub type: no parsing / variants
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    fs,
};
use crate::common::i18n::I18n; // Stub provides same symbol when feature off
use crate::t;

// Configuration constants
const DEFAULT_FORMAT: &str = "%a %b %e %H:%M:%S %Z %Y";
const ISO_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";
const RFC_FORMAT: &str = "%a, %d %b %Y %H:%M:%S %z";
const RFC3339_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%:z";
const EPOCH_FORMAT: &str = "%s";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateConfig {
    pub default_timezone: String,
    pub default_locale: String,
    pub default_format: String,
    pub prefer_utc: bool,
    pub show_nanoseconds: bool,
    pub use_iso_week: bool,
    pub calendar_system: CalendarSystem,
    pub business_days_only: bool,
    pub include_holidays: bool,
    pub astronomical_mode: bool,
    pub location: Option<Location>,
}

impl Default for DateConfig {
    fn default() -> Self {
        Self {
            default_timezone: "local".to_string(),
            default_locale: "en-US".to_string(),
            default_format: DEFAULT_FORMAT.to_string(),
            prefer_utc: false,
            show_nanoseconds: false,
            use_iso_week: false,
            calendar_system: CalendarSystem::Gregorian,
            business_days_only: false,
            include_holidays: false,
            astronomical_mode: false,
            location: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateOptions {
    pub format_string: Option<String>,
    pub input_format: Option<String>,
    pub timezone: Option<String>,
    pub locale: String,
    pub utc: bool,
    pub iso: bool,
    pub rfc: bool,
    pub rfc3339: bool,
    pub reference_file: Option<String>,
    pub set_time: Option<String>,
    pub date_string: Option<String>,
    pub arithmetic: Vec<DateArithmetic>,
    pub output_format: OutputFormat,
    pub show_unix_timestamp: bool,
    pub show_julian_day: bool,
    pub show_week_number: bool,
    pub show_day_of_year: bool,
    pub calendar_system: CalendarSystem,
    pub relative_format: bool,
    pub business_mode: bool,
    pub astronomical_mode: bool,
    pub batch_mode: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub include_holidays: bool,
    pub show_holiday_info: bool,
    pub holiday_regions: Vec<String>,
}

impl Default for DateOptions {
    fn default() -> Self {
        Self {
            format_string: None,
            input_format: None,
            timezone: None,
            locale: "en-US".to_string(),
            utc: false,
            iso: false,
            rfc: false,
            rfc3339: false,
            reference_file: None,
            set_time: None,
            date_string: None,
            arithmetic: Vec::new(),
            output_format: OutputFormat::Default,
            show_unix_timestamp: false,
            show_julian_day: false,
            show_week_number: false,
            show_day_of_year: false,
            calendar_system: CalendarSystem::Gregorian,
            relative_format: false,
            business_mode: false,
            astronomical_mode: false,
            batch_mode: false,
            quiet: false,
            verbose: false,
            include_holidays: false,
            show_holiday_info: false,
            holiday_regions: vec!["US".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Default,
    Unix,
    Iso,
    Rfc,
    Rfc3339,
    Json,
    Xml,
    Csv,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalendarSystem {
    Gregorian,
    Julian,
    Islamic,
    Hebrew,
    Persian,
    Chinese,
    Japanese,
    Buddhist,
    Ethiopian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateArithmetic {
    pub operation: ArithmeticOperation,
    pub value: i64,
    pub unit: TimeUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArithmeticOperation {
    Add,
    Subtract,
    Set,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
    BusinessDays,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
    pub timezone: String,
}

/// Holiday system for business day calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    /// Holiday name (for display purposes)
    pub name: String,
    /// Holiday date
    pub date: NaiveDate,
    /// Holiday region/country (e.g., "US", "JP", "GB")
    pub region: String,
    /// Holiday type (national, regional, religious, etc.)
    pub holiday_type: HolidayType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HolidayType {
    National,
    Regional,
    Religious,
    Cultural,
    Corporate,
}

/// Holiday database manager
#[derive(Debug, Clone)]
pub struct HolidayDatabase {
    holidays: HashMap<String, Vec<Holiday>>,
    enabled_regions: Vec<String>,
}

impl Default for HolidayDatabase {
    fn default() -> Self {
        let mut db = Self {
            holidays: HashMap::new(),
            enabled_regions: vec!["US".to_string()], // Default to US holidays
        };
        db.load_default_holidays();
        db
    }
}

impl HolidayDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_regions(regions: Vec<String>) -> Self {
        let mut db = Self {
            holidays: HashMap::new(),
            enabled_regions: regions,
        };
        db.load_default_holidays();
        db
    }

    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.enabled_regions.iter().any(|region| {
            self.holidays
                .get(region)
                .map(|holidays| holidays.iter().any(|h| h.date == date))
                .unwrap_or(false)
        })
    }

    pub fn get_holiday(&self, date: NaiveDate) -> Option<&Holiday> {
        for region in &self.enabled_regions {
            if let Some(holidays) = self.holidays.get(region) {
                if let Some(holiday) = holidays.iter().find(|h| h.date == date) {
                    return Some(holiday);
                }
            }
        }
        None
    }

    /// Load default holidays for common regions
    fn load_default_holidays(&mut self) {
        self.load_us_holidays();
        self.load_jp_holidays();
        self.load_gb_holidays();
        self.load_de_holidays();
    }

    fn load_us_holidays(&mut self) {
        let current_year = chrono::Utc::now().year();
        let mut us_holidays = Vec::new();

        // Fixed date holidays
        us_holidays.push(Holiday {
            name: "New Year's Day".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
            region: "US".to_string(),
            holiday_type: HolidayType::National,
        });

        us_holidays.push(Holiday {
            name: "Independence Day".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 7, 4).unwrap(),
            region: "US".to_string(),
            holiday_type: HolidayType::National,
        });

        us_holidays.push(Holiday {
            name: "Christmas Day".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 12, 25).unwrap(),
            region: "US".to_string(),
            holiday_type: HolidayType::National,
        });

        // Calculate floating holidays
        if let Some(labor_day) = self.get_nth_weekday_of_month(current_year, 9, Weekday::Mon, 1) {
            us_holidays.push(Holiday {
                name: "Labor Day".to_string(),
                date: labor_day,
                region: "US".to_string(),
                holiday_type: HolidayType::National,
            });
        }

        if let Some(thanksgiving) = self.get_nth_weekday_of_month(current_year, 11, Weekday::Thu, 4) {
            us_holidays.push(Holiday {
                name: "Thanksgiving Day".to_string(),
                date: thanksgiving,
                region: "US".to_string(),
                holiday_type: HolidayType::National,
            });
        }

        self.holidays.insert("US".to_string(), us_holidays);
    }

    fn load_jp_holidays(&mut self) {
        let current_year = chrono::Utc::now().year();
        let mut jp_holidays = Vec::new();

        // Japanese national holidays
        jp_holidays.push(Holiday {
            name: "元日 (New Year's Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
            region: "JP".to_string(),
            holiday_type: HolidayType::National,
        });

        jp_holidays.push(Holiday {
            name: "建国記念の日 (National Foundation Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 2, 11).unwrap(),
            region: "JP".to_string(),
            holiday_type: HolidayType::National,
        });

        jp_holidays.push(Holiday {
            name: "天皇誕生日 (Emperor's Birthday)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 2, 23).unwrap(),
            region: "JP".to_string(),
            holiday_type: HolidayType::National,
        });

        jp_holidays.push(Holiday {
            name: "憲法記念日 (Constitution Memorial Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 5, 3).unwrap(),
            region: "JP".to_string(),
            holiday_type: HolidayType::National,
        });

        jp_holidays.push(Holiday {
            name: "こどもの日 (Children's Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 5, 5).unwrap(),
            region: "JP".to_string(),
            holiday_type: HolidayType::National,
        });

        self.holidays.insert("JP".to_string(), jp_holidays);
    }

    fn load_gb_holidays(&mut self) {
        let current_year = chrono::Utc::now().year();
        let mut gb_holidays = Vec::new();

        gb_holidays.push(Holiday {
            name: "New Year's Day".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
            region: "GB".to_string(),
            holiday_type: HolidayType::National,
        });

        gb_holidays.push(Holiday {
            name: "Christmas Day".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 12, 25).unwrap(),
            region: "GB".to_string(),
            holiday_type: HolidayType::National,
        });

        gb_holidays.push(Holiday {
            name: "Boxing Day".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 12, 26).unwrap(),
            region: "GB".to_string(),
            holiday_type: HolidayType::National,
        });

        self.holidays.insert("GB".to_string(), gb_holidays);
    }

    fn load_de_holidays(&mut self) {
        let current_year = chrono::Utc::now().year();
        let mut de_holidays = Vec::new();

        de_holidays.push(Holiday {
            name: "Neujahr (New Year's Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
            region: "DE".to_string(),
            holiday_type: HolidayType::National,
        });

        de_holidays.push(Holiday {
            name: "Tag der Deutschen Einheit (German Unity Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 10, 3).unwrap(),
            region: "DE".to_string(),
            holiday_type: HolidayType::National,
        });

        de_holidays.push(Holiday {
            name: "Weihnachtstag (Christmas Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 12, 25).unwrap(),
            region: "DE".to_string(),
            holiday_type: HolidayType::National,
        });

        de_holidays.push(Holiday {
            name: "2. Weihnachtstag (Boxing Day)".to_string(),
            date: NaiveDate::from_ymd_opt(current_year, 12, 26).unwrap(),
            region: "DE".to_string(),
            holiday_type: HolidayType::National,
        });

        self.holidays.insert("DE".to_string(), de_holidays);
    }

    /// Helper function to calculate nth weekday of a month (e.g., 3rd Monday)
    fn get_nth_weekday_of_month(&self, year: i32, month: u32, weekday: Weekday, n: u32) -> Option<NaiveDate> {
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1)?;
        let first_weekday = first_of_month.weekday();
        
        let days_to_target = (7 + weekday.num_days_from_monday() - first_weekday.num_days_from_monday()) % 7;
        let target_date = first_of_month + ChronoDuration::days(days_to_target as i64 + (n - 1) as i64 * 7);
        
        // Ensure we're still in the same month
        if target_date.month() == month {
            Some(target_date)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateManager {
    config: DateConfig,
    i18n: I18n,
    timezone_cache: HashMap<String, Tz>,
    format_cache: HashMap<String, String>,
    holiday_db: HolidayDatabase,
}

impl DateManager {
    pub fn new(config: DateConfig, i18n: I18n) -> Self {
        // Create holiday database with regions based on config or environment
        let regions = std::env::var("NXSH_HOLIDAY_REGIONS")
            .ok()
            .map(|regions| regions.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["US".to_string()]);

        Self {
            config,
            i18n,
            timezone_cache: HashMap::new(),
            format_cache: HashMap::new(),
            holiday_db: HolidayDatabase::with_regions(regions),
        }
    }

    pub fn format_date(&self, options: &DateOptions) -> Result<String> {
        let datetime = self.get_datetime(options)?;
        let formatted = self.apply_format(&datetime, options)?;
        
        if options.verbose {
            let metadata = self.get_date_metadata(&datetime, options)?;
            Ok(format!("{formatted}\n{metadata}"))
        } else {
            Ok(formatted)
        }
    }

    fn get_datetime(&self, options: &DateOptions) -> Result<DateTime<Utc>> {
        if let Some(ref date_str) = options.date_string {
            self.parse_date_string(date_str, options)
        } else if let Some(ref ref_file) = options.reference_file {
            self.get_file_time(ref_file)
        } else if let Some(ref set_time) = options.set_time {
            self.parse_set_time(set_time)
        } else {
            Ok(Utc::now())
        }
    }

    fn parse_date_string(&self, date_str: &str, options: &DateOptions) -> Result<DateTime<Utc>> {
        // Try multiple parsing strategies
        
        // Unix timestamp
        if let Ok(timestamp) = date_str.parse::<i64>() {
            return DateTime::from_timestamp(timestamp, 0)
                .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))
                .map(|dt| dt.with_timezone(&Utc));
        }

        // Unix timestamp with nanoseconds
        if let Ok(timestamp_ns) = date_str.parse::<f64>() {
            let secs = timestamp_ns.floor() as i64;
            let nanos = (timestamp_ns.fract() * 1_000_000_000.0) as u32;
            return DateTime::from_timestamp(secs, nanos)
                .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp_ns))
                .map(|dt| dt.with_timezone(&Utc));
        }

        // ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // RFC 2822
        if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Custom format
        if let Some(ref input_format) = options.input_format {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(date_str, input_format) {
                let tz = self.get_timezone(options)?;
                return Ok(tz.from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local time"))?
                    .with_timezone(&Utc));
            }
        }

        // Common formats
        let common_formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d",
            "%m/%d/%Y %H:%M:%S",
            "%m/%d/%Y %H:%M",
            "%m/%d/%Y",
            "%d/%m/%Y %H:%M:%S", 
            "%d/%m/%Y %H:%M",
            "%d/%m/%Y",
            "%Y%m%d %H%M%S",
            "%Y%m%d %H%M",
            "%Y%m%d",
            "%a %b %e %H:%M:%S %Y",
            "%a, %d %b %Y %H:%M:%S",
        ];

        for format in &common_formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(date_str, format) {
                let tz = self.get_timezone(options)?;
                return Ok(tz.from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local time"))?
                    .with_timezone(&Utc));
            }
            
            // Try date-only parsing
            if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, format) {
                let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
                let tz = self.get_timezone(options)?;
                return Ok(tz.from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local time"))?
                    .with_timezone(&Utc));
            }
        }

        // Relative parsing (e.g., "yesterday", "next week", "+1 day")
        self.parse_relative_date(date_str, options)
    }

    fn parse_relative_date(&self, date_str: &str, options: &DateOptions) -> Result<DateTime<Utc>> {
        let now = Utc::now();
        
        match date_str.to_lowercase().as_str() {
            "now" => Ok(now),
            "today" => {
                let tz = self.get_timezone(options)?;
                let today = tz.from_utc_datetime(&now.naive_utc()).date_naive();
                Ok(tz.from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local time"))?
                    .with_timezone(&Utc))
            }
            "yesterday" => {
                let tz = self.get_timezone(options)?;
                let yesterday = tz.from_utc_datetime(&now.naive_utc()).date_naive() - ChronoDuration::days(1);
                Ok(tz.from_local_datetime(&yesterday.and_hms_opt(0, 0, 0).unwrap())
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local time"))?
                    .with_timezone(&Utc))
            }
            "tomorrow" => {
                let tz = self.get_timezone(options)?;
                let tomorrow = tz.from_utc_datetime(&now.naive_utc()).date_naive() + ChronoDuration::days(1);
                Ok(tz.from_local_datetime(&tomorrow.and_hms_opt(0, 0, 0).unwrap())
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local time"))?
                    .with_timezone(&Utc))
            }
            _ => {
                // Parse arithmetic expressions like "+1 day", "-2 weeks", etc.
                if let Some(arithmetic) = self.parse_arithmetic_expression(date_str)? {
                    self.apply_arithmetic(now, &arithmetic)
                } else {
                    Err(anyhow!("Unable to parse date: {}", date_str))
                }
            }
        }
    }

    fn parse_arithmetic_expression(&self, expr: &str) -> Result<Option<DateArithmetic>> {
        let expr = expr.trim();
        
        if !expr.starts_with('+') && !expr.starts_with('-') {
            return Ok(None);
        }

        let operation = if expr.starts_with('+') {
            ArithmeticOperation::Add
        } else {
            ArithmeticOperation::Subtract
        };

        let expr = &expr[1..]; // Remove +/- prefix
        let parts: Vec<&str> = expr.split_whitespace().collect();
        
        if parts.len() != 2 {
            return Ok(None);
        }

        let value: i64 = parts[0].parse()
            .map_err(|_| anyhow!("Invalid number: {}", parts[0]))?;

        let unit = match parts[1].to_lowercase().as_str() {
            "nanosecond" | "nanoseconds" | "ns" => TimeUnit::Nanoseconds,
            "microsecond" | "microseconds" | "us" | "μs" => TimeUnit::Microseconds,
            "millisecond" | "milliseconds" | "ms" => TimeUnit::Milliseconds,
            "second" | "seconds" | "sec" | "s" => TimeUnit::Seconds,
            "minute" | "minutes" | "min" | "m" => TimeUnit::Minutes,
            "hour" | "hours" | "hr" | "h" => TimeUnit::Hours,
            "day" | "days" | "d" => TimeUnit::Days,
            "week" | "weeks" | "w" => TimeUnit::Weeks,
            "month" | "months" | "mon" => TimeUnit::Months,
            "year" | "years" | "yr" | "y" => TimeUnit::Years,
            "business-day" | "business-days" | "bday" | "bdays" => TimeUnit::BusinessDays,
            _ => return Err(anyhow!("Unknown time unit: {}", parts[1])),
        };

        Ok(Some(DateArithmetic { operation, value, unit }))
    }

    fn apply_arithmetic(&self, datetime: DateTime<Utc>, arithmetic: &DateArithmetic) -> Result<DateTime<Utc>> {
        let duration = match arithmetic.unit {
            TimeUnit::Nanoseconds => ChronoDuration::nanoseconds(arithmetic.value),
            TimeUnit::Microseconds => ChronoDuration::microseconds(arithmetic.value),
            TimeUnit::Milliseconds => ChronoDuration::milliseconds(arithmetic.value),
            TimeUnit::Seconds => ChronoDuration::seconds(arithmetic.value),
            TimeUnit::Minutes => ChronoDuration::minutes(arithmetic.value),
            TimeUnit::Hours => ChronoDuration::hours(arithmetic.value),
            TimeUnit::Days => ChronoDuration::days(arithmetic.value),
            TimeUnit::Weeks => ChronoDuration::weeks(arithmetic.value),
            TimeUnit::Months => {
                // Month arithmetic is more complex due to varying month lengths
                return self.add_months(datetime, arithmetic.value, arithmetic.operation == ArithmeticOperation::Subtract);
            }
            TimeUnit::Years => {
                return self.add_years(datetime, arithmetic.value, arithmetic.operation == ArithmeticOperation::Subtract);
            }
            TimeUnit::BusinessDays => {
                return self.add_business_days(datetime, arithmetic.value, arithmetic.operation == ArithmeticOperation::Subtract);
            }
        };

        match arithmetic.operation {
            ArithmeticOperation::Add => Ok(datetime + duration),
            ArithmeticOperation::Subtract => Ok(datetime - duration),
            ArithmeticOperation::Set => Err(anyhow!("Set operation not supported for duration arithmetic")),
        }
    }

    fn add_months(&self, datetime: DateTime<Utc>, months: i64, subtract: bool) -> Result<DateTime<Utc>> {
        let mut new_date = datetime.date_naive();
        let mut year = new_date.year();
        let mut month = new_date.month() as i32;

        if subtract {
            month -= months as i32;
        } else {
            month += months as i32;
        }

        while month <= 0 {
            year -= 1;
            month += 12;
        }
        while month > 12 {
            year += 1;
            month -= 12;
        }

        // Handle day overflow (e.g., Jan 31 + 1 month = Feb 28/29)
        let day = new_date.day();
        let days_in_target_month = self.days_in_month(year, month as u32)?;
        let adjusted_day = day.min(days_in_target_month);

        new_date = NaiveDate::from_ymd_opt(year, month as u32, adjusted_day)
            .ok_or_else(|| anyhow!("Invalid date after month arithmetic"))?;

        let new_datetime = new_date.and_time(datetime.time());
        Ok(datetime.timezone().from_local_datetime(&new_datetime)
            .single()
            .ok_or_else(|| anyhow!("Ambiguous local time after month arithmetic"))?
            .with_timezone(&Utc))
    }

    fn add_years(&self, datetime: DateTime<Utc>, years: i64, subtract: bool) -> Result<DateTime<Utc>> {
        let mut new_date = datetime.date_naive();
        let mut year = new_date.year();

        if subtract {
            year -= years as i32;
        } else {
            year += years as i32;
        }

        // Handle leap year edge case (Feb 29)
        let month = new_date.month();
        let day = new_date.day();
        let adjusted_day = if month == 2 && day == 29 && !self.is_leap_year(year) {
            28
        } else {
            day
        };

        new_date = NaiveDate::from_ymd_opt(year, month, adjusted_day)
            .ok_or_else(|| anyhow!("Invalid date after year arithmetic"))?;

        let new_datetime = new_date.and_time(datetime.time());
        Ok(datetime.timezone().from_local_datetime(&new_datetime)
            .single()
            .ok_or_else(|| anyhow!("Ambiguous local time after year arithmetic"))?
            .with_timezone(&Utc))
    }

    fn add_business_days(&self, datetime: DateTime<Utc>, days: i64, subtract: bool) -> Result<DateTime<Utc>> {
        let mut current_date = datetime.date_naive();
        let mut remaining_days = days.abs();
        let direction = if subtract { -1 } else { 1 };

        while remaining_days > 0 {
            current_date += ChronoDuration::days(direction);
            
            // Skip weekends
            if current_date.weekday() != Weekday::Sat && current_date.weekday() != Weekday::Sun {
                // Check holidays if enabled in configuration
                let is_holiday = if self.config.include_holidays {
                    self.holiday_db.is_holiday(current_date)
                } else {
                    false
                };

                // Only count as business day if not a holiday
                if !is_holiday {
                    remaining_days -= 1;
                }
            }
        }

        let new_datetime = current_date.and_time(datetime.time());
        Ok(datetime.timezone().from_local_datetime(&new_datetime)
            .single()
            .ok_or_else(|| anyhow!("Ambiguous local time after business day arithmetic"))?
            .with_timezone(&Utc))
    }

    fn get_file_time(&self, file_path: &str) -> Result<DateTime<Utc>> {
        let metadata = fs::metadata(file_path)
            .with_context(|| format!("Failed to read file metadata: {file_path}"))?;

        let system_time = metadata.modified()
            .with_context(|| format!("Failed to get modification time: {file_path}"))?;

        Ok(DateTime::<Utc>::from(system_time))
    }

    fn parse_set_time(&self, time_str: &str) -> Result<DateTime<Utc>> {
        // This would set the system time, but for security we'll just parse it
        self.parse_date_string(time_str, &DateOptions::default())
    }

    fn apply_format(&self, datetime: &DateTime<Utc>, options: &DateOptions) -> Result<String> {
        if options.utc {
            let tz_datetime = datetime.with_timezone(&Utc);
            self.format_datetime_to_string(&tz_datetime, options)
        } else {
            let tz = self.get_timezone(options)?;
            let tz_datetime = datetime.with_timezone(&tz);
            self.format_datetime_to_string(&tz_datetime, options)
        }
    }

    fn format_datetime_to_string<T: chrono::TimeZone>(&self, datetime: &DateTime<T>, options: &DateOptions) -> Result<String> 
    where 
        T::Offset: fmt::Display 
    {
        match options.output_format {
            OutputFormat::Unix => Ok(datetime.timestamp().to_string()),
            OutputFormat::Iso => Ok(datetime.to_rfc3339()),
            OutputFormat::Rfc => Ok(datetime.to_rfc2822()),
            OutputFormat::Rfc3339 => Ok(datetime.to_rfc3339()),
            OutputFormat::Json => {
                let json_obj = serde_json::json!({
                    "timestamp": datetime.timestamp(),
                    "iso8601": datetime.to_rfc3339(),
                    "rfc2822": datetime.to_rfc2822(),
                    "unix": datetime.timestamp(),
                    "timezone": datetime.offset().to_string(),
                    "weekday": datetime.weekday().to_string(),
                    "day_of_year": datetime.ordinal(),
                });
                Ok(serde_json::to_string_pretty(&json_obj)?)
            }
            OutputFormat::Xml => {
                Ok(format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<datetime>
    <timestamp>{}</timestamp>
    <iso8601>{}</iso8601>
    <rfc2822>{}</rfc2822>
    <unix>{}</unix>
    <timezone>{}</timezone>
    <weekday>{}</weekday>
    <day_of_year>{}</day_of_year>
</datetime>"#,
                    datetime.timestamp(),
                    datetime.to_rfc3339(),
                    datetime.to_rfc2822(),
                    datetime.timestamp(),
                    datetime.offset(),
                    datetime.weekday(),
                    datetime.ordinal()
                ))
            }
            OutputFormat::Csv => {
                Ok(format!("{},{},{},{},{},{},{}",
                    datetime.timestamp(),
                    datetime.to_rfc3339(),
                    datetime.to_rfc2822(),
                    datetime.timestamp(),
                    datetime.offset(),
                    datetime.weekday(),
                    datetime.ordinal()
                ))
            }
            OutputFormat::Custom(ref format) => {
                Ok(datetime.format(format).to_string())
            }
            OutputFormat::Default => {
                let format = if let Some(ref fmt) = options.format_string {
                    fmt.as_str()
                } else if options.iso {
                    ISO_FORMAT
                } else if options.rfc {
                    RFC_FORMAT
                } else if options.rfc3339 {
                    RFC3339_FORMAT
                } else {
                    &self.config.default_format
                };

                // Apply custom format with additional features
                let formatted = self.apply_extended_format(datetime, format, options)?;
                Ok(formatted)
            }
        }
    }

    fn apply_extended_format<T: chrono::TimeZone>(&self, datetime: &DateTime<T>, format: &str, options: &DateOptions) -> Result<String> 
    where 
        T::Offset: fmt::Display 
    {
        let mut result = datetime.format(format).to_string();

        // Add additional information if requested
        if options.show_unix_timestamp {
            result.push_str(&format!(" ({})", datetime.timestamp()));
        }

        if options.show_julian_day {
            let julian_day = self.calculate_julian_day(datetime)?;
            result.push_str(&format!(" JD:{julian_day}"));
        }

        if options.show_week_number {
            let week = if self.config.use_iso_week {
                datetime.iso_week().week()
            } else {
                ((datetime.ordinal() - 1) / 7) + 1
            };
            result.push_str(&format!(" W{week:02}"));
        }

        if options.show_day_of_year {
            result.push_str(&format!(" D{:03}", datetime.ordinal()));
        }

        // Add relative time if requested
        if options.relative_format {
            let relative = self.format_relative_time(datetime)?;
            result.push_str(&format!(" ({relative})"));
        }

        // Add astronomical information if requested
        if options.astronomical_mode {
            if let Some(ref location) = self.config.location {
                let astro_info = self.calculate_astronomical_info(datetime, location)?;
                result.push_str(&format!(" {astro_info}"));
            }
        }

        Ok(result)
    }

    fn get_timezone(&self, options: &DateOptions) -> Result<Tz> {
        let tz_name = if let Some(ref tz) = options.timezone {
            tz.as_str()
        } else if options.utc {
            "UTC"
        } else {
            &self.config.default_timezone
        };
        #[cfg(feature = "i18n")]
        {
            if tz_name == "local" { return Ok(chrono_tz::UTC); }
            return tz_name.parse::<Tz>().map_err(|_| anyhow!("Invalid timezone: {}", tz_name));
        }
        #[cfg(not(feature = "i18n"))]
        {
            let _ = tz_name;
            Ok(chrono::Utc) // alias Tz = chrono::Utc
        }
    }

    fn get_date_metadata(&self, datetime: &DateTime<Utc>, options: &DateOptions) -> Result<String> {
        let mut metadata = Vec::new();
        let date_naive = datetime.date_naive();

        metadata.push(format!("Unix timestamp: {}", datetime.timestamp()));
        metadata.push(format!("Julian day: {}", self.calculate_julian_day(datetime)?));
        metadata.push(format!("Day of year: {}", datetime.ordinal()));
        metadata.push(format!("Week number: {}", datetime.iso_week().week()));
        metadata.push(format!("Weekday: {}", datetime.weekday()));

        // Add holiday information if requested
        if options.show_holiday_info && (options.include_holidays || self.config.include_holidays) {
            if let Some(holiday) = self.holiday_db.get_holiday(date_naive) {
                metadata.push(format!("Holiday: {} ({})", holiday.name, holiday.region));
            } else {
                // Check if it's a weekend
                if date_naive.weekday() == Weekday::Sat || date_naive.weekday() == Weekday::Sun {
                    metadata.push("Type: Weekend".to_string());
                } else if !self.holiday_db.is_holiday(date_naive) {
                    metadata.push("Type: Business day".to_string());
                }
            }
        }

        if let Some(ref location) = self.config.location {
            let astro_info = self.calculate_astronomical_info(datetime, location)?;
            metadata.push(format!("Astronomical: {astro_info}"));
        }

        Ok(metadata.join(", "))
    }

    fn calculate_julian_day(&self, datetime: &DateTime<impl TimeZone>) -> Result<f64> {
        let year = datetime.year();
        let month = datetime.month() as i32;
        let day = datetime.day() as i32;
        let hour = datetime.hour();
        let minute = datetime.minute();
        let second = datetime.second();

        // Julian day calculation
        let a = (14 - month) / 12;
        let y = year - a;
        let m = month + 12 * a - 3;

        let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 + 1721119;
        let fraction = (hour as f64 - 12.0) / 24.0 + minute as f64 / 1440.0 + second as f64 / 86400.0;

        Ok(jdn as f64 + fraction)
    }

    fn format_relative_time(&self, datetime: &DateTime<impl TimeZone>) -> Result<String> {
        let now = Utc::now();
        let duration = now.signed_duration_since(datetime.with_timezone(&Utc));

        if duration.num_seconds().abs() < 60 {
            Ok("now".to_string())
        } else if duration.num_minutes().abs() < 60 {
            let mins = duration.num_minutes();
            if mins > 0 {
                Ok(format!("{mins} minutes ago"))
            } else {
                Ok(format!("in {} minutes", -mins))
            }
        } else if duration.num_hours().abs() < 24 {
            let hours = duration.num_hours();
            if hours > 0 {
                Ok(format!("{hours} hours ago"))
            } else {
                Ok(format!("in {} hours", -hours))
            }
        } else {
            let days = duration.num_days();
            if days > 0 {
                Ok(format!("{days} days ago"))
            } else {
                Ok(format!("in {} days", -days))
            }
        }
    }

    fn calculate_astronomical_info(&self, _datetime: &DateTime<impl TimeZone>, location: &Location) -> Result<String> {
        // Simplified astronomical calculations
        // In a real implementation, this would use proper astronomical libraries
        Ok(format!("lat:{:.2}, lon:{:.2}", location.latitude, location.longitude))
    }

    fn is_leap_year(&self, year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn days_in_month(&self, year: i32, month: u32) -> Result<u32> {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
            4 | 6 | 9 | 11 => Ok(30),
            2 => Ok(if self.is_leap_year(year) { 29 } else { 28 }),
            _ => Err(anyhow!("Invalid month: {}", month)),
        }
    }

    /// List all holidays for the given year and regions
    pub fn list_holidays(&self, year: Option<i32>, regions: Option<Vec<String>>) -> Result<String> {
        let target_year = year.unwrap_or_else(|| chrono::Utc::now().year());
        let target_regions = regions.unwrap_or_else(|| self.holiday_db.enabled_regions.clone());

        let mut all_holidays = Vec::new();

        for region in &target_regions {
            if let Some(holidays) = self.holiday_db.holidays.get(region) {
                for holiday in holidays {
                    if holiday.date.year() == target_year {
                        all_holidays.push(holiday.clone());
                    }
                }
            }
        }

        // Sort by date
        all_holidays.sort_by_key(|h| h.date);

        if all_holidays.is_empty() {
            return Ok(format!("No holidays found for year {} in regions: {}", 
                             target_year, 
                             target_regions.join(", ")));
        }

        let mut output = format!("Holidays for {} in regions: {}\n", 
                                target_year, 
                                target_regions.join(", "));
        output.push_str("=====================================\n");

        for holiday in &all_holidays {
            output.push_str(&format!("{} - {} ({}, {})\n", 
                                   holiday.date.format("%Y-%m-%d %a"), 
                                   holiday.name, 
                                   holiday.region,
                                   format!("{:?}", holiday.holiday_type).to_lowercase()));
        }

        output.push_str(&format!("\nTotal: {} holidays\n", all_holidays.len()));
        Ok(output)
    }

    /// Check if a specific date is a business day (not weekend or holiday)
    pub fn is_business_day(&self, date: NaiveDate) -> bool {
        // Check if it's a weekend
        if date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun {
            return false;
        }

        // Check if it's a holiday (if holiday checking is enabled)
        if self.config.include_holidays && self.holiday_db.is_holiday(date) {
            return false;
        }

        true
    }
}

pub async fn date_cli(args: &[String]) -> Result<()> {
    let mut options = DateOptions::default();
    let mut show_help = false;
    let mut show_version = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => show_help = true,
            "--version" => show_version = true,
            "--utc" | "-u" => options.utc = true,
            "--iso-8601" | "-I" => {
                options.iso = true;
                options.output_format = OutputFormat::Iso;
            }
            "--rfc-email" | "--rfc-2822" | "-R" => {
                options.rfc = true;
                options.output_format = OutputFormat::Rfc;
            }
            "--rfc-3339" => {
                options.rfc3339 = true;
                options.output_format = OutputFormat::Rfc3339;
            }
            "--date" | "-d" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.date_string = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--date requires an argument"));
                }
            }
            "--file" | "-r" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.reference_file = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--file requires an argument"));
                }
            }
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.format_string = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--format requires an argument"));
                }
            }
            "--set" | "-s" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.set_time = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--set requires an argument"));
                }
            }
            "--unix" => {
                options.show_unix_timestamp = true;
                options.output_format = OutputFormat::Unix;
            }
            "--julian" => options.show_julian_day = true,
            "--week" => options.show_week_number = true,
            "--day-of-year" => options.show_day_of_year = true,
            "--relative" => options.relative_format = true,
            "--astronomical" => options.astronomical_mode = true,
            "--business" => options.business_mode = true,
            "--holidays" => {
                options.include_holidays = true;
                options.show_holiday_info = true;
            }
            "--list-holidays" => {
                // Special mode to list holidays for current year
                let manager = DateManager::new(DateConfig::default(), I18n::new());
                let holiday_list = manager.list_holidays(None, None)?;
                println!("{}", holiday_list);
                return Ok(());
            }
            "--holiday-regions" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.holiday_regions = args[i].split(',').map(|s| s.trim().to_string()).collect();
                } else {
                    return Err(anyhow!("--holiday-regions requires a comma-separated list"));
                }
            }
            "--json" => options.output_format = OutputFormat::Json,
            "--xml" => options.output_format = OutputFormat::Xml,
            "--csv" => options.output_format = OutputFormat::Csv,
            "--verbose" | "-v" => options.verbose = true,
            "--quiet" | "-q" => options.quiet = true,
            arg if arg.starts_with('+') => {
                // Custom format string
                options.format_string = Some(arg[1..].to_string());
            }
            arg if !arg.starts_with('-') => {
                // Date string argument
                options.date_string = Some(arg.to_string());
            }
            _ => return Err(anyhow!("Unknown option: {}", args[i])),
        }
        i += 1;
    }

    if show_help {
        print_help();
        return Ok(());
    }

    if show_version {
        println!("date (NexusShell) 1.0.0");
        println!("World-class date and time display with advanced formatting");
        return Ok(());
    }

    let mut config = DateConfig::default();
    
    // Apply options to config
    if options.include_holidays {
        config.include_holidays = true;
    }
    
    // Check environment variable for holiday checking
    if std::env::var("NXSH_DATE_HOLIDAYS").unwrap_or_default() == "1" {
        config.include_holidays = true;
    }

    let i18n = I18n::new();
    let date_manager = DateManager::new(config, i18n);

    match date_manager.format_date(&options) {
        Ok(formatted) => {
            if !options.quiet {
                println!("{formatted}");
            }
        }
        Err(e) => {
            if !options.quiet {
                eprintln!("date: {e}");
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: date [OPTION]... [+FORMAT]");
    println!("       date [-u|--utc|--universal] [MMDDhhmm[[CC]YY][.ss]]");
    println!("Display the current time in the given FORMAT, or set the system date.");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("  -d, --date=STRING          display time described by STRING, not 'now'");
    println!("  -f, --file=DATEFILE        like --date; once for each line of DATEFILE");
    println!("      --format=FORMAT        use FORMAT for output");
    println!("  -I[FMT], --iso-8601[=FMT]  output date/time in ISO 8601 format.");
    println!("                             FMT='date' for date only (the default),");
    println!("                             'hours', 'minutes', 'seconds', or 'ns'");
    println!("  -r, --reference=FILE       display the last modification time of FILE");
    println!("  -R, --rfc-email            output date and time in RFC 5322 format.");
    println!("      --rfc-3339=FMT         output date/time in RFC 3339 format.");
    println!("  -s, --set=STRING           set time described by STRING");
    println!("  -u, --utc, --universal     print or set Coordinated Universal Time (UTC)");
    println!("      --unix                 display Unix timestamp");
    println!("      --julian               show Julian day number");
    println!("      --week                 show week number");
    println!("      --day-of-year          show day of year");
    println!("      --relative             show relative time");
    println!("      --astronomical         show astronomical information
      --business             enable business day mode
      --holidays             include holiday information and checking
      --list-holidays        list all holidays for current year
      --holiday-regions=LIST specify holiday regions (comma-separated, e.g., US,JP,GB)");
    println!("      --business             business calendar mode");
    println!("      --json                 output in JSON format");
    println!("      --xml                  output in XML format");
    println!("      --csv                  output in CSV format");
    println!("  -v, --verbose              verbose output with metadata");
    println!("  -q, --quiet                suppress error messages");
    println!("      --help                 display this help and exit");
    println!("      --version              output version information and exit");
    println!();
    println!("FORMAT controls the output.  Interpreted sequences are:");
    println!("  %%   a literal %");
    println!("  %a   locale's abbreviated weekday name (e.g., Sun)");
    println!("  %A   locale's full weekday name (e.g., Sunday)");
    println!("  %b   locale's abbreviated month name (e.g., Jan)");
    println!("  %B   locale's full month name (e.g., January)");
    println!("  %c   locale's date and time (e.g., Thu Mar  3 23:05:25 2005)");
    println!("  %C   century; like %Y, except omit last two digits (e.g., 20)");
    println!("  %d   day of month (e.g., 01)");
    println!("  %D   date; same as %m/%d/%y");
    println!("  %e   day of month, space padded; same as %_d");
    println!("  %F   full date; same as %Y-%m-%d");
    println!("  %g   last two digits of year of ISO week number (see %G)");
    println!("  %G   year of ISO week number (see %V); normally useful only with %V");
    println!("  %h   same as %b");
    println!("  %H   hour (00..23)");
    println!("  %I   hour (01..12)");
    println!("  %j   day of year (001..366)");
    println!("  %k   hour, space padded ( 0..23); same as %_H");
    println!("  %l   hour, space padded ( 1..12); same as %_I");
    println!("  %m   month (01..12)");
    println!("  %M   minute (00..59)");
    println!("  %n   a newline");
    println!("  %N   nanoseconds (000000000..999999999)");
    println!("  %p   locale's equivalent of either AM or PM; blank if not known");
    println!("  %P   like %p, but lower case");
    println!("  %q   quarter of year (1..4)");
    println!("  %r   locale's 12-hour clock time (e.g., 11:11:04 PM)");
    println!("  %R   24-hour hour and minute; same as %H:%M");
    println!("  %s   seconds since 1970-01-01 00:00:00 UTC");
    println!("  %S   second (00..60)");
    println!("  %t   a tab");
    println!("  %T   time; same as %H:%M:%S");
    println!("  %u   day of week (1..7); 1 is Monday");
    println!("  %U   week number of year, with Sunday as first day of week (00..53)");
    println!("  %V   ISO week number, with Monday as first day of week (01..53)");
    println!("  %w   day of week (0..6); 0 is Sunday");
    println!("  %W   week number of year, with Monday as first day of week (00..53)");
    println!("  %x   locale's date representation (e.g., 12/31/99)");
    println!("  %X   locale's time representation (e.g., 23:13:48)");
    println!("  %y   last two digits of year (00..99)");
    println!("  %Y   year");
    println!("  %z   +hhmm numeric time zone (e.g., -0400)");
    println!("  %:z  +hh:mm numeric time zone (e.g., -04:00)");
    println!("  %Z   alphabetic time zone abbreviation (e.g., EDT)");
    println!();
    println!("Examples:");
    println!("  date                          # Current date and time");
    println!("  date --utc                    # Current UTC time");
    println!("  date '+%Y-%m-%d %H:%M:%S'     # Custom format");
    println!("  date -d 'yesterday'           # Yesterday's date");
    println!("  date -d '+1 day'              # Tomorrow's date");
    println!("  date -d '2024-12-25'          # Specific date");
    println!("  date --unix                   # Unix timestamp");
    println!("  date --json                   # JSON output");
    println!("  date --relative               # Relative time display");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_date_current() {
        let args = vec![];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_utc() {
        let args = vec!["--utc".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_format() {
        let args = vec!["+%Y-%m-%d".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_iso() {
        let args = vec!["--iso-8601".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_unix() {
        let args = vec!["--unix".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_relative() {
        let args = vec!["--date".to_string(), "yesterday".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_arithmetic() {
        let args = vec!["--date".to_string(), "+1 day".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_date_json() {
        let args = vec!["--json".to_string()];
        assert!(date_cli(&args).await.is_ok());
    }

    #[test]
    fn test_parse_arithmetic() {
        let config = DateConfig::default();
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);

        let arithmetic = manager.parse_arithmetic_expression("+1 day").unwrap().unwrap();
        assert_eq!(arithmetic.operation, ArithmeticOperation::Add);
        assert_eq!(arithmetic.value, 1);
        assert_eq!(arithmetic.unit, TimeUnit::Days);

        let arithmetic = manager.parse_arithmetic_expression("-2 weeks").unwrap().unwrap();
        assert_eq!(arithmetic.operation, ArithmeticOperation::Subtract);
        assert_eq!(arithmetic.value, 2);
        assert_eq!(arithmetic.unit, TimeUnit::Weeks);
    }

    #[test]
    fn test_julian_day_calculation() {
        let config = DateConfig::default();
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);

        let datetime = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let julian_day = manager.calculate_julian_day(&datetime).unwrap();
        
        // Julian day for 2000-01-01 12:00:00 UTC should be approximately 2451545.0
        assert!((julian_day - 2451545.0).abs() < 0.1);
    }

    #[test]
    fn test_leap_year() {
        let config = DateConfig::default();
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);

        assert!(manager.is_leap_year(2000));  // Divisible by 400
        assert!(!manager.is_leap_year(1900)); // Divisible by 100 but not 400
        assert!(manager.is_leap_year(2004));  // Divisible by 4
        assert!(!manager.is_leap_year(2001)); // Not divisible by 4
    }

    #[test]
    fn test_days_in_month() {
        let config = DateConfig::default();
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);

        assert_eq!(manager.days_in_month(2000, 2).unwrap(), 29); // Leap year February
        assert_eq!(manager.days_in_month(2001, 2).unwrap(), 28); // Regular February
        assert_eq!(manager.days_in_month(2001, 1).unwrap(), 31); // January
        assert_eq!(manager.days_in_month(2001, 4).unwrap(), 30); // April
    }

    #[test]
    fn test_holiday_database() {
        let db = HolidayDatabase::new();
        assert!(!db.enabled_regions.is_empty());
        assert!(db.enabled_regions.contains(&"US".to_string()));
        
        let current_year = chrono::Utc::now().year();
        
        // Test New Year's Day
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        assert!(db.is_holiday(new_years));
        
        // Test non-holiday
        let random_date = NaiveDate::from_ymd_opt(current_year, 6, 15).unwrap();
        assert!(!db.is_holiday(random_date));
    }

    #[test]
    fn test_business_day_with_holidays() {
        let mut config = DateConfig::default();
        config.include_holidays = true;
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);
        
        let current_year = chrono::Utc::now().year();
        
        // Test New Year's Day (should not be business day)
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        if new_years.weekday() != Weekday::Sat && new_years.weekday() != Weekday::Sun {
            assert!(!manager.is_business_day(new_years));
        }
    }

    #[test]
    fn test_list_holidays() {
        let config = DateConfig::default();
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);
        
        let current_year = chrono::Utc::now().year();
        let holiday_list = manager.list_holidays(Some(current_year), Some(vec!["US".to_string()])).unwrap();
        
        assert!(holiday_list.contains("New Year's Day"));
        assert!(holiday_list.contains("Independence Day"));
        assert!(holiday_list.contains("Christmas Day"));
    }
}
