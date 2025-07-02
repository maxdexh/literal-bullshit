use std::{
    collections::HashMap,
    fmt::Display,
    num::NonZeroU32,
    ops::{Mul, Range},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::{bail, ensure};
use num::BigUint;

pub type BookingId = u64;
pub type CustomerId = u64;
pub type RoomId = u64;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct HotelId(NonZeroU32);
impl HotelId {
    const MAX_DIGITS: usize = 5;
}
impl FromStr for HotelId {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = NonZeroU32::from_str(s)?;
        if id.to_string().len() > Self::MAX_DIGITS {
            bail!("Hotel id {id} must not exceed {}", Self::MAX_DIGITS)
        }
        Ok(Self(id))
    }
}
impl Display for HotelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{padding}{num}",
            num = self.0,
            padding = "0".repeat(Self::MAX_DIGITS - self.0.to_string().len())
        )
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Price {
    pub cents: num::BigUint,
}
impl Price {
    const UNIT: char = '€';
    const SEPARATOR: char = '.';
    const MAX_SMALL_DIGITS: usize = 2;
    const SMALL_TO_BIG: u32 = u32::pow(10, Self::MAX_SMALL_DIGITS as u32);
}

impl FromStr for Price {
    type Err = anyhow::Error;
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(Self::UNIT) {
            s = &s[..s.len() - 1];
        }
        let mut parts = s.split(Self::SEPARATOR);

        let [lhs, rhs] = std::array::from_fn(|_| parts.next().unwrap_or(""));

        ensure!(
            parts.next().is_none(),
            "Cannot have multiple decimal seperators in price",
        );
        let big = BigUint::from_str(lhs)? * Self::SMALL_TO_BIG;
        let small = match rhs.len() {
            0 => 0,
            1..Self::MAX_SMALL_DIGITS => {
                let dist = (Self::MAX_SMALL_DIGITS - rhs.len()) as _;
                u32::from_str(rhs)? * u32::pow(10, dist)
            }
            Self::MAX_SMALL_DIGITS.. => {
                let (decimal, zeros) = rhs.split_at(Self::MAX_SMALL_DIGITS);
                ensure!(
                    zeros.is_empty() || BigUint::from_str(zeros)? == BigUint::ZERO,
                    "Price only has {} digits of precision",
                    Self::MAX_SMALL_DIGITS,
                );

                u32::from_str(decimal)?
            }
        };
        let cents = big + small;
        ensure!(cents != BigUint::ZERO, "Price must be non-zero");
        Ok(Self { cents })
    }
}
impl Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{big}{sep}{small}{unit}",
            big = &self.cents / Self::SMALL_TO_BIG,
            sep = Self::SEPARATOR,
            small = &self.cents % Self::SMALL_TO_BIG,
            unit = Self::UNIT,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Single,
    Double,
    Suite,
}
impl FromStr for Category {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Single" => Self::Single,
            "Double" => Self::Double,
            "Suite" => Self::Suite,
            _ => bail!("Unknown category '{s}'"),
        })
    }
}
impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Single => "Single",
                Self::Double => "Double",
                Self::Suite => "Suite",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(time::Date);
impl Date {
    pub fn into_inner(self) -> time::Date {
        let Self(date) = self;
        date
    }
}

static DATE_FORMAT: LazyLock<time::format_description::OwnedFormatItem> =
    LazyLock::new(|| time::format_description::parse_owned::<2>("[year]-[month]-[day]").unwrap());
impl FromStr for Date {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(time::Date::parse(s, &DATE_FORMAT)?))
    }
}
impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(&DATE_FORMAT).unwrap())
    }
}

pub struct HotelData {
    pub city: String,
    pub rooms: HashMap<u64, RoomData>,
}
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Person {
    pub forename: String,
    pub surname: String,
}

pub struct RoomData {
    pub category: Category,
    pub price: Price,
    pub bookings: Vec<Booking>,
}

pub struct Booking {
    pub time: Range<Date>,
    pub customer: CustomerId,
    pub id: BookingId,
}

impl RoomData {
    pub(crate) fn is_occupied(&self, Range { start, end }: Range<&Date>) -> bool {
        for Booking { time, .. } in &self.bookings {
            if time.contains(start) || time.contains(end) {
                return true;
            }
        }
        false
    }
}
