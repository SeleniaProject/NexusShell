use chrono::{NaiveDate, Weekday};
use crate::date::{HolidayDatabase, Holiday, HolidayType};

#[cfg(test)]
mod holiday_tests {
    use super::*;

    #[test]
    fn test_holiday_database_creation() {
        let db = HolidayDatabase::new();
        assert!(!db.enabled_regions.is_empty());
        assert!(db.enabled_regions.contains(&"US".to_string()));
    }

    #[test]
    fn test_us_holidays() {
        let db = HolidayDatabase::with_regions(vec!["US".to_string()]);
        let current_year = chrono::Utc::now().year();
        
        // Test New Year's Day
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        assert!(db.is_holiday(new_years));
        
        // Test Independence Day
        let july_4th = NaiveDate::from_ymd_opt(current_year, 7, 4).unwrap();
        assert!(db.is_holiday(july_4th));
        
        // Test Christmas
        let christmas = NaiveDate::from_ymd_opt(current_year, 12, 25).unwrap();
        assert!(db.is_holiday(christmas));
        
        // Test non-holiday
        let random_date = NaiveDate::from_ymd_opt(current_year, 6, 15).unwrap();
        assert!(!db.is_holiday(random_date));
    }

    #[test]
    fn test_japanese_holidays() {
        let db = HolidayDatabase::with_regions(vec!["JP".to_string()]);
        let current_year = chrono::Utc::now().year();
        
        // Test New Year's Day (å…E—¥)
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        assert!(db.is_holiday(new_years));
        
        // Test Children's Day (ã“ã©ã‚‚ãEæ—¥)
        let childrens_day = NaiveDate::from_ymd_opt(current_year, 5, 5).unwrap();
        assert!(db.is_holiday(childrens_day));
    }

    #[test]
    fn test_multiple_regions() {
        let db = HolidayDatabase::with_regions(vec!["US".to_string(), "JP".to_string()]);
        let current_year = chrono::Utc::now().year();
        
        // Test New Year's Day (holiday in both US and JP)
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        assert!(db.is_holiday(new_years));
        
        // Get holiday info
        let holiday = db.get_holiday(new_years).unwrap();
        assert_eq!(holiday.name, "New Year's Day");
    }

    #[test]
    fn test_holiday_get_info() {
        let db = HolidayDatabase::with_regions(vec!["US".to_string()]);
        let current_year = chrono::Utc::now().year();
        
        let july_4th = NaiveDate::from_ymd_opt(current_year, 7, 4).unwrap();
        
        if let Some(holiday) = db.get_holiday(july_4th) {
            assert_eq!(holiday.name, "Independence Day");
            assert_eq!(holiday.region, "US");
            assert_eq!(holiday.holiday_type, HolidayType::National);
        } else {
            panic!("Independence Day should be found");
        }
    }

    #[test]
    fn test_nth_weekday_calculation() {
        let db = HolidayDatabase::new();
        let current_year = chrono::Utc::now().year();
        
        // Test Labor Day (1st Monday of September)
        if let Some(labor_day) = db.get_nth_weekday_of_month(current_year, 9, Weekday::Mon, 1) {
            assert_eq!(labor_day.weekday(), Weekday::Mon);
            assert_eq!(labor_day.month(), 9);
            assert!(labor_day.day() <= 7); // First week
        }
        
        // Test Thanksgiving (4th Thursday of November)
        if let Some(thanksgiving) = db.get_nth_weekday_of_month(current_year, 11, Weekday::Thu, 4) {
            assert_eq!(thanksgiving.weekday(), Weekday::Thu);
            assert_eq!(thanksgiving.month(), 11);
            assert!(thanksgiving.day() >= 22 && thanksgiving.day() <= 28); // 4th week
        }
    }

    #[test]
    fn test_no_holiday_regions() {
        let db = HolidayDatabase::with_regions(vec!["XX".to_string()]); // Non-existent region
        let current_year = chrono::Utc::now().year();
        
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        assert!(!db.is_holiday(new_years)); // Should not find holiday in non-existent region
    }
}

#[cfg(test)]
mod date_business_day_tests {
    use super::*;
    use crate::date::{DateConfig, DateManager};
    use crate::common::i18n::I18n;

    #[test]
    fn test_business_day_with_holidays_disabled() {
        let config = DateConfig {
            include_holidays: false,
            ..Default::default()
        };
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);
        
        let current_year = chrono::Utc::now().year();
        
        // Test weekday (should be business day)
        let monday = NaiveDate::from_ymd_opt(current_year, 6, 3).unwrap(); // Assuming this is a Monday
        let weekday = monday.weekday();
        if weekday != Weekday::Sat && weekday != Weekday::Sun {
            assert!(manager.is_business_day(monday));
        }
        
        // Test weekend (should not be business day)
        let sunday = NaiveDate::from_ymd_opt(current_year, 6, 2).unwrap(); // Assuming this is a Sunday
        if sunday.weekday() == Weekday::Sun {
            assert!(!manager.is_business_day(sunday));
        }
    }

    #[test]
    fn test_business_day_with_holidays_enabled() {
        let config = DateConfig {
            include_holidays: true,
            ..Default::default()
        };
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);
        
        let current_year = chrono::Utc::now().year();
        
        // Test New Year's Day (should not be business day when holidays enabled)
        let new_years = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        if new_years.weekday() != Weekday::Sat && new_years.weekday() != Weekday::Sun {
            assert!(!manager.is_business_day(new_years));
        }
        
        // Test regular weekday that's not a holiday (should be business day)
        let regular_tuesday = NaiveDate::from_ymd_opt(current_year, 6, 4).unwrap(); // Assuming this is a Tuesday
        if regular_tuesday.weekday() == Weekday::Tue && !manager.holiday_db.is_holiday(regular_tuesday) {
            assert!(manager.is_business_day(regular_tuesday));
        }
    }

    #[test]
    fn test_list_holidays_functionality() {
        let config = DateConfig::default();
        let i18n = I18n::new();
        let manager = DateManager::new(config, i18n);
        
        let current_year = chrono::Utc::now().year();
        let holiday_list = manager.list_holidays(Some(current_year), Some(vec!["US".to_string()])).unwrap();
        
        // Should contain some holidays
        assert!(holiday_list.contains("New Year's Day"));
        assert!(holiday_list.contains("Independence Day"));
        assert!(holiday_list.contains("Christmas Day"));
        assert!(holiday_list.contains(&format!("Holidays for {}", current_year)));
    }
}

