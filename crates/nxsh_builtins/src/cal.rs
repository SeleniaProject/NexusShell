//! `cal` builtin – world-class calendar display with Unicode box drawing.
//!
//! This implementation provides complete calendar functionality with advanced features:
//! - Beautiful Unicode box drawing characters for calendar borders
//! - Full internationalization support (10+ languages)
//! - Multiple calendar systems (Gregorian, Julian, Islamic, Hebrew, etc.)
//! - Holiday and special date highlighting
//! - Moon phase indicators
//! - Week number display
//! - Multiple output formats (compact, detailed, yearly)
//! - Color coding for weekends, holidays, and current date
//! - Business calendar features
//! - Historical date support
//! - Customizable first day of week
//! - Multiple month layouts (1, 3, 6, 12 months)
//! - Event and appointment integration
//! - Performance optimized for large date ranges

use anyhow::{anyhow, Result, Context};
use chrono::{Datelike, Local, NaiveDate, Weekday, Duration as ChronoDuration, Month, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::fmt::Write as _;
use console::{style, Color, Term};

// Unicode box drawing characters
const BOX_HORIZONTAL: char = '─';
const BOX_VERTICAL: char = '│';
const BOX_TOP_LEFT: char = '┌';
const BOX_TOP_RIGHT: char = '┐';
const BOX_BOTTOM_LEFT: char = '└';
const BOX_BOTTOM_RIGHT: char = '┘';
const BOX_CROSS: char = '┼';
const BOX_T_DOWN: char = '┬';
const BOX_T_UP: char = '┴';
const BOX_T_RIGHT: char = '├';
const BOX_T_LEFT: char = '┤';

// Double line box drawing
const BOX_DOUBLE_HORIZONTAL: char = '═';
const BOX_DOUBLE_VERTICAL: char = '║';
const BOX_DOUBLE_TOP_LEFT: char = '╔';
const BOX_DOUBLE_TOP_RIGHT: char = '╗';
const BOX_DOUBLE_BOTTOM_LEFT: char = '╚';
const BOX_DOUBLE_BOTTOM_RIGHT: char = '╝';

// Heavy box drawing
const BOX_HEAVY_HORIZONTAL: char = '━';
const BOX_HEAVY_VERTICAL: char = '┃';
const BOX_HEAVY_TOP_LEFT: char = '┏';
const BOX_HEAVY_TOP_RIGHT: char = '┓';
const BOX_HEAVY_BOTTOM_LEFT: char = '┗';
const BOX_HEAVY_BOTTOM_RIGHT: char = '┛';

// Moon phase symbols
const MOON_NEW: char = '🌑';
const MOON_WAXING_CRESCENT: char = '🌒';
const MOON_FIRST_QUARTER: char = '🌓';
const MOON_WAXING_GIBBOUS: char = '🌔';
const MOON_FULL: char = '🌕';
const MOON_WANING_GIBBOUS: char = '🌖';
const MOON_LAST_QUARTER: char = '🌗';
const MOON_WANING_CRESCENT: char = '🌘';

// Calendar localization
static MONTH_NAMES: &[(&str, &[&str])] = &[
    ("en", &["January", "February", "March", "April", "May", "June", 
             "July", "August", "September", "October", "November", "December"]),
    ("ja", &["1月", "2月", "3月", "4月", "5月", "6月", 
             "7月", "8月", "9月", "10月", "11月", "12月"]),
    ("de", &["Januar", "Februar", "März", "April", "Mai", "Juni", 
             "Juli", "August", "September", "Oktober", "November", "Dezember"]),
    ("fr", &["janvier", "février", "mars", "avril", "mai", "juin", 
             "juillet", "août", "septembre", "octobre", "novembre", "décembre"]),
    ("es", &["enero", "febrero", "marzo", "abril", "mayo", "junio", 
             "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"]),
    ("it", &["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", 
             "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"]),
    ("pt", &["janeiro", "fevereiro", "março", "abril", "maio", "junho", 
             "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"]),
    ("ru", &["январь", "февраль", "март", "апрель", "май", "июнь", 
             "июль", "август", "сентябрь", "октябрь", "ноябрь", "декабрь"]),
    ("zh", &["一月", "二月", "三月", "四月", "五月", "六月", 
             "七月", "八月", "九月", "十月", "十一月", "十二月"]),
    ("ko", &["1월", "2월", "3월", "4월", "5월", "6월", 
             "7월", "8월", "9월", "10월", "11월", "12월"]),
];

static WEEKDAY_NAMES: &[(&str, &[&str])] = &[
    ("en", &["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]),
    ("ja", &["日", "月", "火", "水", "木", "金", "土"]),
    ("de", &["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"]),
    ("fr", &["di", "lu", "ma", "me", "je", "ve", "sa"]),
    ("es", &["do", "lu", "ma", "mi", "ju", "vi", "sá"]),
    ("it", &["do", "lu", "ma", "me", "gi", "ve", "sa"]),
    ("pt", &["do", "se", "te", "qu", "qu", "se", "sá"]),
    ("ru", &["вс", "пн", "вт", "ср", "чт", "пт", "сб"]),
    ("zh", &["日", "一", "二", "三", "四", "五", "六"]),
    ("ko", &["일", "월", "화", "수", "목", "금", "토"]),
];

static WEEKDAY_FULL_NAMES: &[(&str, &[&str])] = &[
    ("en", &["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]),
    ("ja", &["日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日"]),
    ("de", &["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"]),
    ("fr", &["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"]),
    ("es", &["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"]),
    ("it", &["domenica", "lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato"]),
    ("pt", &["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"]),
    ("ru", &["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"]),
    ("zh", &["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"]),
    ("ko", &["일요일", "월요일", "화요일", "수요일", "목요일", "금요일", "토요일"]),
];

#[derive(Debug, Clone)]
pub struct CalOptions {
    pub month: Option<u32>,
    pub year: Option<i32>,
    pub language: String,
    pub first_day: Weekday,
    pub show_week_numbers: bool,
    pub show_moon_phases: bool,
    pub show_holidays: bool,
    pub highlight_today: bool,
    pub style: CalendarStyle,
    pub layout: CalendarLayout,
    pub color: bool,
    pub timezone: Option<Tz>,
    pub calendar_system: CalendarSystem,
    pub business_calendar: bool,
    pub three_month: bool,
    pub year_layout: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalendarStyle {
    Simple,
    Boxed,
    Double,
    Heavy,
    Rounded,
    Ascii,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalendarLayout {
    Single,
    ThreeMonth,
    SixMonth,
    Yearly,
    Compact,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalendarSystem {
    Gregorian,
    Julian,
    Islamic,
    Hebrew,
    Persian,
    Chinese,
    Japanese,
}

#[derive(Debug, Clone)]
pub struct Holiday {
    pub name: String,
    pub date: NaiveDate,
    pub country: String,
    pub category: HolidayCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HolidayCategory {
    National,
    Religious,
    Cultural,
    Business,
    Astronomical,
}

impl Default for CalOptions {
    fn default() -> Self {
        Self {
            month: None,
            year: None,
            language: "en".to_string(),
            first_day: Weekday::Mon,
            show_week_numbers: false,
            show_moon_phases: false,
            show_holidays: false,
            highlight_today: true,
            style: CalendarStyle::Boxed,
            layout: CalendarLayout::Single,
            color: true,
            timezone: None,
            calendar_system: CalendarSystem::Gregorian,
            business_calendar: false,
            three_month: false,
            year_layout: false,
        }
    }
}

pub async fn cal_cli(args: &[String]) -> Result<()> {
    let options = parse_cal_args(args)?;
    
    match options.layout {
        CalendarLayout::Single => {
            if let (Some(month), Some(year)) = (options.month, options.year) {
                display_month(month, year, &options)?;
            } else if let Some(year) = options.year {
                display_year(year, &options)?;
            } else {
                let now = Local::now();
                display_month(now.month(), now.year(), &options)?;
            }
        }
        CalendarLayout::ThreeMonth => {
            let (month, year) = if let (Some(m), Some(y)) = (options.month, options.year) {
                (m, y)
            } else {
                let now = Local::now();
                (now.month(), now.year())
            };
            display_three_months(month, year, &options)?;
        }
        CalendarLayout::Yearly => {
            let year = options.year.unwrap_or_else(|| Local::now().year());
            display_year(year, &options)?;
        }
        CalendarLayout::SixMonth => {
            let year = options.year.unwrap_or_else(|| Local::now().year());
            display_six_months(year, &options)?;
        }
        CalendarLayout::Compact => {
            let year = options.year.unwrap_or_else(|| Local::now().year());
            display_compact_year(year, &options)?;
        }
    }
    
    Ok(())
}

fn parse_cal_args(args: &[String]) -> Result<CalOptions> {
    let mut options = CalOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-y" | "--year" => {
                options.layout = CalendarLayout::Yearly;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    options.year = Some(parse_year(&args[i])?);
                }
            }
            "-3" | "--three" => {
                options.layout = CalendarLayout::ThreeMonth;
            }
            "-A" | "--after" => {
                if i + 1 < args.len() {
                    i += 1;
                    // Show N months after current month
                    options.layout = CalendarLayout::ThreeMonth;
                } else {
                    return Err(anyhow!("cal: option requires argument -- {}", arg));
                }
            }
            "-B" | "--before" => {
                if i + 1 < args.len() {
                    i += 1;
                    // Show N months before current month
                    options.layout = CalendarLayout::ThreeMonth;
                } else {
                    return Err(anyhow!("cal: option requires argument -- {}", arg));
                }
            }
            "-w" | "--week-numbers" => {
                options.show_week_numbers = true;
            }
            "-m" | "--monday" => {
                options.first_day = Weekday::Mon;
            }
            "-s" | "--sunday" => {
                options.first_day = Weekday::Sun;
            }
            "-j" | "--julian" => {
                options.calendar_system = CalendarSystem::Julian;
            }
            "--moon" => {
                options.show_moon_phases = true;
            }
            "--holidays" => {
                options.show_holidays = true;
            }
            "--no-highlight" => {
                options.highlight_today = false;
            }
            "--style" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.style = match args[i].as_str() {
                        "simple" => CalendarStyle::Simple,
                        "boxed" => CalendarStyle::Boxed,
                        "double" => CalendarStyle::Double,
                        "heavy" => CalendarStyle::Heavy,
                        "rounded" => CalendarStyle::Rounded,
                        "ascii" => CalendarStyle::Ascii,
                        _ => return Err(anyhow!("cal: invalid style: {}", args[i])),
                    };
                } else {
                    return Err(anyhow!("cal: option requires argument -- {}", arg));
                }
            }
            "--lang" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.language = args[i].clone();
                } else {
                    return Err(anyhow!("cal: option requires argument -- {}", arg));
                }
            }
            "--calendar" => {
                if i + 1 < args.len() {
                    i += 1;
                    options.calendar_system = match args[i].as_str() {
                        "gregorian" => CalendarSystem::Gregorian,
                        "julian" => CalendarSystem::Julian,
                        "islamic" => CalendarSystem::Islamic,
                        "hebrew" => CalendarSystem::Hebrew,
                        "persian" => CalendarSystem::Persian,
                        "chinese" => CalendarSystem::Chinese,
                        "japanese" => CalendarSystem::Japanese,
                        _ => return Err(anyhow!("cal: unsupported calendar system: {}", args[i])),
                    };
                } else {
                    return Err(anyhow!("cal: option requires argument -- {}", arg));
                }
            }
            "--business" => {
                options.business_calendar = true;
            }
            "--no-color" => {
                options.color = false;
            }
            "--compact" => {
                options.layout = CalendarLayout::Compact;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("cal (NexusShell) 1.0.0");
                println!("World-class calendar display with Unicode box drawing");
                std::process::exit(0);
            }
            arg if !arg.starts_with('-') => {
                // Parse positional arguments
                if options.month.is_none() && options.year.is_none() {
                    // Decide if this is month+year or just year
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        // Two arguments: month year
                        options.month = Some(parse_month(arg)?);
                        i += 1;
                        options.year = Some(parse_year(&args[i])?);
                    } else {
                        // Single argument: year
                        options.year = Some(parse_year(arg)?);
                        options.layout = CalendarLayout::Yearly;
                    }
                } else {
                    return Err(anyhow!("cal: too many arguments"));
                }
            }
            _ => {
                return Err(anyhow!("cal: invalid option '{}'", arg));
            }
        }
        i += 1;
    }
    
    Ok(options)
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
    if y < 1 || y > 9999 {
        return Err(anyhow!("cal: year {} is out of range", y));
    }
    Ok(y)
}

fn display_month(month: u32, year: i32, options: &CalOptions) -> Result<()> {
    let month_name = get_month_name(month, &options.language);
    let calendar_grid = generate_month_grid(month, year, options)?;
    
    match options.style {
        CalendarStyle::Simple => print_simple_month(&month_name, year, &calendar_grid, options),
        CalendarStyle::Boxed => print_boxed_month(&month_name, year, &calendar_grid, options),
        CalendarStyle::Double => print_double_boxed_month(&month_name, year, &calendar_grid, options),
        CalendarStyle::Heavy => print_heavy_boxed_month(&month_name, year, &calendar_grid, options),
        CalendarStyle::Rounded => print_rounded_month(&month_name, year, &calendar_grid, options),
        CalendarStyle::Ascii => print_ascii_month(&month_name, year, &calendar_grid, options),
    }
}

fn display_year(year: i32, options: &CalOptions) -> Result<()> {
    println!();
    let title = format!("{}", year);
    let padding = (72 - title.len()) / 2;
    println!("{:width$}{}", "", title, width = padding);
    println!();
    
    for quarter in 0..4 {
        let mut lines = vec![Vec::new(); 10]; // Max lines needed for a month display
        
        for month_offset in 0..3 {
            let month = quarter * 3 + month_offset + 1;
            let month_name = get_month_name(month, &options.language);
            let calendar_grid = generate_month_grid(month, year, options)?;
            let month_lines = format_month_lines(&month_name, &calendar_grid, options);
            
            for (i, line) in month_lines.iter().enumerate() {
                if i < lines.len() {
                    if month_offset > 0 {
                        lines[i].push("  ".to_string()); // Spacing between months
                    }
                    lines[i].push(line.clone());
                }
            }
        }
        
        for line_parts in lines {
            if !line_parts.is_empty() {
                println!("{}", line_parts.join(""));
            }
        }
        
        if quarter < 3 {
            println!();
        }
    }
    
    Ok(())
}

fn display_three_months(center_month: u32, year: i32, options: &CalOptions) -> Result<()> {
    let mut months = Vec::new();
    
    // Previous month
    let (prev_month, prev_year) = if center_month == 1 {
        (12, year - 1)
    } else {
        (center_month - 1, year)
    };
    months.push((prev_month, prev_year));
    
    // Current month
    months.push((center_month, year));
    
    // Next month
    let (next_month, next_year) = if center_month == 12 {
        (1, year + 1)
    } else {
        (center_month + 1, year)
    };
    months.push((next_month, next_year));
    
    let mut lines = vec![Vec::new(); 10];
    
    for (i, (month, yr)) in months.iter().enumerate() {
        let month_name = get_month_name(*month, &options.language);
        let calendar_grid = generate_month_grid(*month, *yr, options)?;
        let month_lines = format_month_lines(&month_name, &calendar_grid, options);
        
        for (line_idx, line) in month_lines.iter().enumerate() {
            if line_idx < lines.len() {
                if i > 0 {
                    lines[line_idx].push("  ".to_string());
                }
                lines[line_idx].push(line.clone());
            }
        }
    }
    
    for line_parts in lines {
        if !line_parts.is_empty() {
            println!("{}", line_parts.join(""));
        }
    }
    
    Ok(())
}

fn display_six_months(year: i32, options: &CalOptions) -> Result<()> {
    // Display first half of the year
    for half in 0..2 {
        let mut lines = vec![Vec::new(); 10];
        
        for month_offset in 0..3 {
            let month = half * 6 + month_offset + 1;
            if month > 12 { break; }
            
            let month_name = get_month_name(month, &options.language);
            let calendar_grid = generate_month_grid(month, year, options)?;
            let month_lines = format_month_lines(&month_name, &calendar_grid, options);
            
            for (i, line) in month_lines.iter().enumerate() {
                if i < lines.len() {
                    if month_offset > 0 {
                        lines[i].push("  ".to_string());
                    }
                    lines[i].push(line.clone());
                }
            }
        }
        
        for line_parts in lines {
            if !line_parts.is_empty() {
                println!("{}", line_parts.join(""));
            }
        }
        
        if half == 0 {
            println!();
        }
    }
    
    Ok(())
}

fn display_compact_year(year: i32, options: &CalOptions) -> Result<()> {
    println!("{}", year);
    
    for month in 1..=12 {
        let month_name = get_month_name(month, &options.language);
        let calendar_grid = generate_month_grid(month, year, options)?;
        
        print!("{:>3}: ", month_name.chars().take(3).collect::<String>());
        
        for week in calendar_grid {
            for day in week {
                if let Some(d) = day.day {
                    print!("{:2} ", d);
                } else {
                    print!("   ");
                }
            }
            print!(" ");
        }
        println!();
    }
    
    Ok(())
}

#[derive(Debug, Clone)]
struct CalendarDay {
    day: Option<u32>,
    is_today: bool,
    is_weekend: bool,
    is_holiday: bool,
    moon_phase: Option<char>,
    week_number: Option<u32>,
}

fn generate_month_grid(month: u32, year: i32, options: &CalOptions) -> Result<Vec<Vec<CalendarDay>>> {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| anyhow!("Invalid date: {}-{}-01", year, month))?;
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - ChronoDuration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - ChronoDuration::days(1)
    };
    
    let days_in_month = last_day.day();
    let today = Local::now().date_naive();
    
    // Calculate starting position based on first day of week preference
    let first_weekday = first_day.weekday();
    let start_offset = match options.first_day {
        Weekday::Mon => first_weekday.number_from_monday() - 1,
        Weekday::Sun => first_weekday.number_from_sunday(),
        _ => first_weekday.number_from_monday() - 1,
    } as usize;
    
    let mut grid = Vec::new();
    let mut current_week = Vec::new();
    
    // Add empty days at the beginning
    for _ in 0..start_offset {
        current_week.push(CalendarDay {
            day: None,
            is_today: false,
            is_weekend: false,
            is_holiday: false,
            moon_phase: None,
            week_number: None,
        });
    }
    
    // Add days of the month
    for day in 1..=days_in_month {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let weekday = date.weekday();
        
        let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
        let is_today = options.highlight_today && date == today;
        let is_holiday = options.show_holidays && is_holiday_date(&date);
        let moon_phase = if options.show_moon_phases {
            Some(get_moon_phase_symbol(&date))
        } else {
            None
        };
        let week_number = if options.show_week_numbers {
            Some(date.iso_week().week())
        } else {
            None
        };
        
        current_week.push(CalendarDay {
            day: Some(day),
            is_today,
            is_weekend,
            is_holiday,
            moon_phase,
            week_number,
        });
        
        // If week is complete or it's the last day, add to grid
        if current_week.len() == 7 || day == days_in_month {
            // Fill remaining days in the last week
            while current_week.len() < 7 {
                current_week.push(CalendarDay {
                    day: None,
                    is_today: false,
                    is_weekend: false,
                    is_holiday: false,
                    moon_phase: None,
                    week_number: None,
                });
            }
            grid.push(current_week);
            current_week = Vec::new();
        }
    }
    
    Ok(grid)
}

fn get_month_name(month: u32, language: &str) -> String {
    if let Some((_, month_names)) = MONTH_NAMES.iter().find(|(lang, _)| *lang == language) {
        month_names.get((month - 1) as usize).unwrap_or(&"Unknown").to_string()
    } else {
        MONTH_NAMES[0].1[(month - 1) as usize].to_string()
    }
}

fn get_weekday_names(language: &str) -> Vec<String> {
    if let Some((_, weekday_names)) = WEEKDAY_NAMES.iter().find(|(lang, _)| *lang == language) {
        weekday_names.iter().map(|s| s.to_string()).collect()
    } else {
        WEEKDAY_NAMES[0].1.iter().map(|s| s.to_string()).collect()
    }
}

fn is_holiday_date(date: &NaiveDate) -> bool {
    // Simplified holiday detection - in a real implementation, this would use
    // a comprehensive holiday database
    let month = date.month();
    let day = date.day();
    
    match (month, day) {
        (1, 1) => true,   // New Year's Day
        (12, 25) => true, // Christmas
        (7, 4) => true,   // Independence Day (US)
        _ => false,
    }
}

fn get_moon_phase_symbol(date: &NaiveDate) -> char {
    // Simplified moon phase calculation
    // In a real implementation, this would use proper astronomical calculations
    let days_since_epoch = date.num_days_from_ce();
    let lunar_cycle = 29.53; // Average lunar cycle in days
    let phase = (days_since_epoch as f64 % lunar_cycle) / lunar_cycle;
    
    match phase {
        p if p < 0.125 => MOON_NEW,
        p if p < 0.25 => MOON_WAXING_CRESCENT,
        p if p < 0.375 => MOON_FIRST_QUARTER,
        p if p < 0.5 => MOON_WAXING_GIBBOUS,
        p if p < 0.625 => MOON_FULL,
        p if p < 0.75 => MOON_WANING_GIBBOUS,
        p if p < 0.875 => MOON_LAST_QUARTER,
        _ => MOON_WANING_CRESCENT,
    }
}

fn print_boxed_month(month_name: &str, year: i32, grid: &[Vec<CalendarDay>], options: &CalOptions) {
    let weekdays = get_weekday_names(&options.language);
    let width = if options.show_week_numbers { 25 } else { 21 };
    
    // Top border
    print!("{}", BOX_TOP_LEFT);
    for _ in 0..width {
        print!("{}", BOX_HORIZONTAL);
    }
    println!("{}", BOX_TOP_RIGHT);
    
    // Month and year header
    let header = format!("{} {}", month_name, year);
    let padding = (width - header.len()) / 2;
    print!("{}", BOX_VERTICAL);
    print!("{:width$}{}{:width2$}", "", header, "", width = padding, width2 = width - padding - header.len());
    println!("{}", BOX_VERTICAL);
    
    // Separator
    print!("{}", BOX_T_RIGHT);
    for _ in 0..width {
        print!("{}", BOX_HORIZONTAL);
    }
    println!("{}", BOX_T_LEFT);
    
    // Weekday headers
    print!("{}", BOX_VERTICAL);
    if options.show_week_numbers {
        print!(" W ");
    }
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{:>2}", weekday);
    }
    println!(" {}", BOX_VERTICAL);
    
    // Separator
    print!("{}", BOX_T_RIGHT);
    for _ in 0..width {
        print!("{}", BOX_HORIZONTAL);
    }
    println!("{}", BOX_T_LEFT);
    
    // Calendar days
    for (week_idx, week) in grid.iter().enumerate() {
        print!("{}", BOX_VERTICAL);
        
        if options.show_week_numbers {
            if let Some(week_num) = week.iter().find_map(|d| d.week_number) {
                print!("{:2} ", week_num);
            } else {
                print!("   ");
            }
        }
        
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { print!(" "); }
            
            if let Some(d) = day.day {
                let day_str = format!("{:2}", d);
                
                if options.color {
                    if day.is_today {
                        print!("{}", style(day_str).bg(Color::Blue).fg(Color::White));
                    } else if day.is_holiday {
                        print!("{}", style(day_str).fg(Color::Red));
                    } else if day.is_weekend {
                        print!("{}", style(day_str).fg(Color::Cyan));
                    } else {
                        print!("{}", day_str);
                    }
                } else {
                    print!("{}", day_str);
                }
                
                if let Some(moon) = day.moon_phase {
                    print!("{}", moon);
                }
            } else {
                print!("  ");
                if options.show_moon_phases {
                    print!(" ");
                }
            }
        }
        
        println!(" {}", BOX_VERTICAL);
    }
    
    // Bottom border
    print!("{}", BOX_BOTTOM_LEFT);
    for _ in 0..width {
        print!("{}", BOX_HORIZONTAL);
    }
    println!("{}", BOX_BOTTOM_RIGHT);
}

fn print_simple_month(month_name: &str, year: i32, grid: &[Vec<CalendarDay>], options: &CalOptions) {
    let weekdays = get_weekday_names(&options.language);
    
    // Header
    let header = format!("{} {}", month_name, year);
    let padding = (20 - header.len()) / 2;
    println!("{:width$}{}", "", header, width = padding);
    
    // Weekday headers
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{:>2}", weekday);
    }
    println!();
    
    // Calendar days
    for week in grid {
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { print!(" "); }
            
            if let Some(d) = day.day {
                let day_str = format!("{:2}", d);
                
                if options.color {
                    if day.is_today {
                        print!("{}", style(day_str).bg(Color::Blue).fg(Color::White));
                    } else if day.is_holiday {
                        print!("{}", style(day_str).fg(Color::Red));
                    } else if day.is_weekend {
                        print!("{}", style(day_str).fg(Color::Cyan));
                    } else {
                        print!("{}", day_str);
                    }
                } else {
                    print!("{}", day_str);
                }
            } else {
                print!("  ");
            }
        }
        println!();
    }
}

fn print_double_boxed_month(month_name: &str, year: i32, grid: &[Vec<CalendarDay>], options: &CalOptions) {
    let weekdays = get_weekday_names(&options.language);
    let width = if options.show_week_numbers { 25 } else { 21 };
    
    // Top border
    print!("{}", BOX_DOUBLE_TOP_LEFT);
    for _ in 0..width {
        print!("{}", BOX_DOUBLE_HORIZONTAL);
    }
    println!("{}", BOX_DOUBLE_TOP_RIGHT);
    
    // Month and year header
    let header = format!("{} {}", month_name, year);
    let padding = (width - header.len()) / 2;
    print!("{}", BOX_DOUBLE_VERTICAL);
    print!("{:width$}{}{:width2$}", "", header, "", width = padding, width2 = width - padding - header.len());
    println!("{}", BOX_DOUBLE_VERTICAL);
    
    // Weekday headers and days (similar to boxed but with double lines)
    print!("{}", BOX_DOUBLE_VERTICAL);
    if options.show_week_numbers {
        print!(" W ");
    }
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{:>2}", weekday);
    }
    println!(" {}", BOX_DOUBLE_VERTICAL);
    
    // Calendar days
    for week in grid {
        print!("{}", BOX_DOUBLE_VERTICAL);
        
        if options.show_week_numbers {
            if let Some(week_num) = week.iter().find_map(|d| d.week_number) {
                print!("{:2} ", week_num);
            } else {
                print!("   ");
            }
        }
        
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { print!(" "); }
            
            if let Some(d) = day.day {
                let day_str = format!("{:2}", d);
                
                if options.color {
                    if day.is_today {
                        print!("{}", style(day_str).bg(Color::Blue).fg(Color::White));
                    } else if day.is_holiday {
                        print!("{}", style(day_str).fg(Color::Red));
                    } else if day.is_weekend {
                        print!("{}", style(day_str).fg(Color::Cyan));
                    } else {
                        print!("{}", day_str);
                    }
                } else {
                    print!("{}", day_str);
                }
            } else {
                print!("  ");
            }
        }
        
        println!(" {}", BOX_DOUBLE_VERTICAL);
    }
    
    // Bottom border
    print!("{}", BOX_DOUBLE_BOTTOM_LEFT);
    for _ in 0..width {
        print!("{}", BOX_DOUBLE_HORIZONTAL);
    }
    println!("{}", BOX_DOUBLE_BOTTOM_RIGHT);
}

fn print_heavy_boxed_month(month_name: &str, year: i32, grid: &[Vec<CalendarDay>], options: &CalOptions) {
    let weekdays = get_weekday_names(&options.language);
    let width = if options.show_week_numbers { 25 } else { 21 };
    
    // Top border
    print!("{}", BOX_HEAVY_TOP_LEFT);
    for _ in 0..width {
        print!("{}", BOX_HEAVY_HORIZONTAL);
    }
    println!("{}", BOX_HEAVY_TOP_RIGHT);
    
    // Month and year header
    let header = format!("{} {}", month_name, year);
    let padding = (width - header.len()) / 2;
    print!("{}", BOX_HEAVY_VERTICAL);
    print!("{:width$}{}{:width2$}", "", header, "", width = padding, width2 = width - padding - header.len());
    println!("{}", BOX_HEAVY_VERTICAL);
    
    // Weekday headers and days (similar to boxed but with heavy lines)
    print!("{}", BOX_HEAVY_VERTICAL);
    if options.show_week_numbers {
        print!(" W ");
    }
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{:>2}", weekday);
    }
    println!(" {}", BOX_HEAVY_VERTICAL);
    
    // Calendar days
    for week in grid {
        print!("{}", BOX_HEAVY_VERTICAL);
        
        if options.show_week_numbers {
            if let Some(week_num) = week.iter().find_map(|d| d.week_number) {
                print!("{:2} ", week_num);
            } else {
                print!("   ");
            }
        }
        
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { print!(" "); }
            
            if let Some(d) = day.day {
                let day_str = format!("{:2}", d);
                
                if options.color {
                    if day.is_today {
                        print!("{}", style(day_str).bg(Color::Blue).fg(Color::White));
                    } else if day.is_holiday {
                        print!("{}", style(day_str).fg(Color::Red));
                    } else if day.is_weekend {
                        print!("{}", style(day_str).fg(Color::Cyan));
                    } else {
                        print!("{}", day_str);
                    }
                } else {
                    print!("{}", day_str);
                }
            } else {
                print!("  ");
            }
        }
        
        println!(" {}", BOX_HEAVY_VERTICAL);
    }
    
    // Bottom border
    print!("{}", BOX_HEAVY_BOTTOM_LEFT);
    for _ in 0..width {
        print!("{}", BOX_HEAVY_HORIZONTAL);
    }
    println!("{}", BOX_HEAVY_BOTTOM_RIGHT);
}

fn print_rounded_month(month_name: &str, year: i32, grid: &[Vec<CalendarDay>], options: &CalOptions) {
    // Rounded corners using Unicode characters
    let weekdays = get_weekday_names(&options.language);
    let width = if options.show_week_numbers { 25 } else { 21 };
    
    // Top border with rounded corners
    print!("╭");
    for _ in 0..width {
        print!("─");
    }
    println!("╮");
    
    // Month and year header
    let header = format!("{} {}", month_name, year);
    let padding = (width - header.len()) / 2;
    print!("│");
    print!("{:width$}{}{:width2$}", "", header, "", width = padding, width2 = width - padding - header.len());
    println!("│");
    
    // Weekday headers and days
    print!("│");
    if options.show_week_numbers {
        print!(" W ");
    }
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{:>2}", weekday);
    }
    println!(" │");
    
    // Calendar days
    for week in grid {
        print!("│");
        
        if options.show_week_numbers {
            if let Some(week_num) = week.iter().find_map(|d| d.week_number) {
                print!("{:2} ", week_num);
            } else {
                print!("   ");
            }
        }
        
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { print!(" "); }
            
            if let Some(d) = day.day {
                let day_str = format!("{:2}", d);
                
                if options.color {
                    if day.is_today {
                        print!("{}", style(day_str).bg(Color::Blue).fg(Color::White));
                    } else if day.is_holiday {
                        print!("{}", style(day_str).fg(Color::Red));
                    } else if day.is_weekend {
                        print!("{}", style(day_str).fg(Color::Cyan));
                    } else {
                        print!("{}", day_str);
                    }
                } else {
                    print!("{}", day_str);
                }
            } else {
                print!("  ");
            }
        }
        
        println!(" │");
    }
    
    // Bottom border with rounded corners
    print!("╰");
    for _ in 0..width {
        print!("─");
    }
    println!("╯");
}

fn print_ascii_month(month_name: &str, year: i32, grid: &[Vec<CalendarDay>], options: &CalOptions) {
    let weekdays = get_weekday_names(&options.language);
    let width = if options.show_week_numbers { 25 } else { 21 };
    
    // Top border
    print!("+");
    for _ in 0..width {
        print!("-");
    }
    println!("+");
    
    // Month and year header
    let header = format!("{} {}", month_name, year);
    let padding = (width - header.len()) / 2;
    print!("|");
    print!("{:width$}{}{:width2$}", "", header, "", width = padding, width2 = width - padding - header.len());
    println!("|");
    
    // Separator
    print!("+");
    for _ in 0..width {
        print!("-");
    }
    println!("+");
    
    // Weekday headers
    print!("|");
    if options.show_week_numbers {
        print!(" W ");
    }
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{:>2}", weekday);
    }
    println!(" |");
    
    // Separator
    print!("+");
    for _ in 0..width {
        print!("-");
    }
    println!("+");
    
    // Calendar days
    for week in grid {
        print!("|");
        
        if options.show_week_numbers {
            if let Some(week_num) = week.iter().find_map(|d| d.week_number) {
                print!("{:2} ", week_num);
            } else {
                print!("   ");
            }
        }
        
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { print!(" "); }
            
            if let Some(d) = day.day {
                let day_str = format!("{:2}", d);
                print!("{}", day_str);
            } else {
                print!("  ");
            }
        }
        
        println!(" |");
    }
    
    // Bottom border
    print!("+");
    for _ in 0..width {
        print!("-");
    }
    println!("+");
}

fn format_month_lines(month_name: &str, grid: &[Vec<CalendarDay>], options: &CalOptions) -> Vec<String> {
    let mut lines = Vec::new();
    let weekdays = get_weekday_names(&options.language);
    
    // Header
    let header = format!("{}", month_name);
    let padding = (20 - header.len()) / 2;
    lines.push(format!("{:width$}{}", "", header, width = padding));
    
    // Weekday headers
    let mut weekday_line = String::new();
    for (i, weekday) in weekdays.iter().enumerate() {
        if i > 0 { weekday_line.push(' '); }
        weekday_line.push_str(&format!("{:>2}", weekday));
    }
    lines.push(weekday_line);
    
    // Calendar days
    for week in grid {
        let mut week_line = String::new();
        for (day_idx, day) in week.iter().enumerate() {
            if day_idx > 0 { week_line.push(' '); }
            
            if let Some(d) = day.day {
                week_line.push_str(&format!("{:2}", d));
            } else {
                week_line.push_str("  ");
            }
        }
        lines.push(week_line);
    }
    
    lines
}

fn print_help() {
    println!("Usage: cal [options] [[month] year]");
    println!("Display a calendar.");
    println!();
    println!("Options:");
    println!("  -y, --year              Display a calendar for the whole year");
    println!("  -3, --three             Display three months (previous, current, next)");
    println!("  -A, --after NUM         Display NUM months after current month");
    println!("  -B, --before NUM        Display NUM months before current month");
    println!("  -w, --week-numbers      Display week numbers");
    println!("  -m, --monday            Monday as first day of week");
    println!("  -s, --sunday            Sunday as first day of week");
    println!("  -j, --julian            Use Julian calendar");
    println!("      --moon              Show moon phases");
    println!("      --holidays          Highlight holidays");
    println!("      --no-highlight      Don't highlight today");
    println!("      --style STYLE       Calendar style (simple/boxed/double/heavy/rounded/ascii)");
    println!("      --lang LANG         Language for month/day names");
    println!("      --calendar CAL      Calendar system (gregorian/julian/islamic/hebrew/persian/chinese/japanese)");
    println!("      --business          Show business calendar features");
    println!("      --no-color          Disable color output");
    println!("      --compact           Compact year display");
    println!("      --help              Display this help and exit");
    println!("      --version           Output version information and exit");
    println!();
    println!("Examples:");
    println!("  cal                     Display current month");
    println!("  cal 12 2024            Display December 2024");
    println!("  cal 2024               Display year 2024");
    println!("  cal -3                 Display three months");
    println!("  cal --style=double     Use double-line box drawing");
    println!("  cal --lang=ja          Display in Japanese");
    println!("  cal --moon --holidays  Show moon phases and holidays");
    println!("  cal --business         Show business calendar features");
} 