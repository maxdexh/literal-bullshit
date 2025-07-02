use std::{
    collections::VecDeque,
    fmt::{Display, Write},
    ops::Range,
};

use anyhow::ensure;

use crate::data::{Booking, BookingId, Category, CustomerId, Date, HotelId, Person, Price, RoomId};

use super::*;

pub fn handle(command: &str, model: &mut Model, args: Vec<&str>) -> Result {
    let mut args = VecDeque::from(args);

    fn ensure_n_args<'a, const N: usize>(args: &[&'a str]) -> Result<[&'a str; N]> {
        let len = args.len();
        args.try_into()
            .map_err(|_| anyhow::format_err!("Expected {N} arguments, got {len}"))
    }
    macro_rules! cmd {
        ($func:ident, $($arg:tt)*) => {
            cmd! { @ $func  [$($arg)*] [] [] }
        };
        ( @ $func:ident [$ty:ty $(, $($rest:tt)*)?] [$($args:expr,)*] [$($pats:pat,)*] ) => {
            cmd! { @ $func [$($($rest)*)?] [$($args,)* arg.parse::<$ty>()?, ] [$($pats,)* arg, ] }
        };
        ( @ $func:ident [] [$($args:expr,)*] [$($pats:pat,)*] ) => {{
            let [$($pats,)*] = ensure_n_args(args.as_slices().0)?;
            $func(model, $($args,)*)?.to_string()
        }};
    }

    const TARGET_ROOM: &str = "room";
    const TARGET_HOTEL: &str = "hotel";
    const TARGET_CHEAPEST: &str = "cheapest";
    const TARGET_AVAILABLE: &str = "available";
    const TARGET_BOOKINGS: &str = "bookings";
    const TARGET_ROOMS: &str = "rooms";
    Ok(match command {
        "add" => match ensure_target(args.pop_front(), &[TARGET_ROOM, TARGET_HOTEL])? {
            TARGET_ROOM => cmd!(add_room, _, _, _, _),
            TARGET_HOTEL => cmd!(add_hotel, _, _),
            _ => unreachable!(),
        },
        "remove" => match ensure_target(args.pop_front(), &[TARGET_ROOM, TARGET_HOTEL])? {
            TARGET_ROOM => cmd!(remove_room, _, _),
            TARGET_HOTEL => cmd!(remove_hotel, _),
            _ => unreachable!(),
        },
        "find" => match ensure_target(args.pop_front(), &[TARGET_CHEAPEST, TARGET_AVAILABLE])? {
            TARGET_CHEAPEST => cmd!(find_cheapest, _, _, _, _),
            TARGET_AVAILABLE => cmd!(find_available, _, _, _, _),
            _ => unreachable!(),
        },
        "list" => match ensure_target(args.pop_front(), &[TARGET_ROOMS, TARGET_BOOKINGS])? {
            TARGET_ROOMS => cmd!(list_rooms,),
            TARGET_BOOKINGS => cmd!(list_bookings,),
            _ => unreachable!(),
        },
        "cancel" => cmd!(cancel, _, _),
        "book" => cmd!(book, _, _, _, _, _, _),
        _ => bail!("Unknown command '{command}'"),
    })
}
fn ensure_target<'a>(arg: Option<&'a str>, targets: &[&str]) -> anyhow::Result<&'a str> {
    let target_list = || targets.join(", ");
    let Some(target) = arg else {
        bail!("Missing target, expected one of {}", target_list());
    };
    ensure!(
        targets.contains(&target),
        "Unknwon target {target}, expected one of {}",
        target_list(),
    );
    Ok(target)
}

fn add_hotel(model: &mut Model, id: HotelId, city: String) -> Result<impl Display> {
    model.add_hotel(id, city).map(|_| "OK")
}
fn add_room(
    model: &mut Model,
    hotel: HotelId,
    room: RoomId,
    category: Category,
    price: Price,
) -> Result<impl Display> {
    model.add_room(hotel, room, category, price).map(|_| "OK")
}
fn remove_hotel(model: &mut Model, id: HotelId) -> Result<impl Display> {
    model.remove_hotel(id).map(|_| "OK")
}
fn remove_room(model: &mut Model, hotel: HotelId, id: RoomId) -> Result<impl Display> {
    model.remove_room(hotel, id).map(|_| "OK")
}
fn list_rooms(model: &Model) -> Result<impl Display> {
    let mut output = String::new();
    let mut rooms: Vec<_> = model.rooms().collect();
    rooms.sort_unstable_by_key(|&(hotel, room_id, _)| (hotel, room_id));
    for (hotel, room_id, room_dat) in &rooms {
        writeln!(
            output,
            "{hotel} {room_id} {cat} {price}",
            cat = room_dat.category,
            price = room_dat.price,
        )?;
    }
    if output.ends_with("\n") {
        output.pop();
    }
    Ok(output)
}
fn list_bookings(model: &Model) -> Result<impl Display> {
    let mut output = String::new();
    let mut bookings: Vec<_> = model.bookings().collect();
    bookings.sort_unstable_by_key(|booking| booking.id);
    for Booking {
        time: Range { start, end },
        customer,
        id,
    } in &bookings
    {
        writeln!(output, "{id} {customer} {start} {end}")?;
    }
    if output.ends_with("\n") {
        output.pop();
    }
    Ok(output)
}
fn find_cheapest(
    model: &Model,
    city: String,
    category: Category,
    start: Date,
    end: Date,
) -> Result<impl Display> {
    let min = model
        .avaiable(
            &city,
            category,
            Range {
                start: &start,
                end: &end,
            },
        )?
        .min_by_key(|(hotel, room, price)| (*price, *hotel, *room));

    if let Some((hotel, room, price)) = min {
        let days = (end.into_inner() - start.into_inner()).whole_days();
        let total = Price {
            cents: &price.cents * days as u64,
        };
        Ok(format!("{hotel} {room} {total}"))
    } else {
        Ok(String::new())
    }
}
fn find_available(
    model: &Model,
    city: String,
    category: Category,
    start: Date,
    end: Date,
) -> Result<impl Display> {
    let mut output = String::new();
    let mut rooms: Vec<_> = model
        .avaiable(
            &city,
            category,
            Range {
                start: &start,
                end: &end,
            },
        )?
        .collect();

    rooms.sort_unstable_by_key(|(hotel, room, _)| (*hotel, *room));

    for (hotel, room, price) in &rooms {
        writeln!(output, "{hotel} {room} {price}")?;
    }

    if output.ends_with("\n") {
        output.pop();
    }
    Ok(output)
}
fn book(
    model: &mut Model,
    hotel_id: HotelId,
    room_id: RoomId,
    start: Date,
    end: Date,
    forename: String,
    surname: String,
) -> Result<impl Display> {
    let customer = model.get_customer(Person { forename, surname });
    let id = model.book(hotel_id, room_id, Range { start, end }, customer)?;
    Ok(format!("{id} {customer}"))
}
fn cancel(model: &mut Model, booking: BookingId, customer: CustomerId) -> Result<impl Display> {
    model.cancel(booking, customer)?;
    Ok("OK")
}
