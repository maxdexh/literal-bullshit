use std::{
    collections::{HashMap, hash_map::Entry},
    ops::Range,
};

use anyhow::{bail, ensure};

use crate::data::{
    Booking, BookingId, Category, CustomerId, Date, HotelData, HotelId, Person, Price, RoomData,
};

pub struct Model {
    cur_booking_id: u64,
    cur_customer_id: u64,
    hotels: HashMap<HotelId, HotelData>,
    customers: HashMap<Person, CustomerId>,
}
impl Model {
    pub fn new() -> Self {
        Self {
            cur_booking_id: 1,
            cur_customer_id: 1,
            hotels: Default::default(),
            customers: Default::default(),
        }
    }

    pub fn add_hotel(&mut self, id: HotelId, city: String) -> anyhow::Result<()> {
        match self.hotels.entry(id) {
            Entry::Occupied(_) => bail!("Hotel ID is already in use"),
            Entry::Vacant(vacant) => vacant.insert(HotelData {
                city,
                rooms: Default::default(),
            }),
        };
        Ok(())
    }
    pub fn add_room(
        &mut self,
        hotel_id: HotelId,
        room_id: u64,
        category: Category,
        price: Price,
    ) -> anyhow::Result<()> {
        let Some(hotel_data) = self.hotels.get_mut(&hotel_id) else {
            bail!("Hotel with id {hotel_id} does not exist")
        };
        match hotel_data.rooms.entry(room_id) {
            Entry::Occupied(_) => bail!("Hotel ID is already in use"),
            Entry::Vacant(vacant) => vacant.insert(RoomData {
                category,
                price,
                bookings: Default::default(),
            }),
        };
        Ok(())
    }
    pub fn remove_room(&mut self, hotel_id: HotelId, room_id: u64) -> anyhow::Result<()> {
        self.hotels
            .get_mut(&hotel_id)
            .ok_or_else(|| anyhow::format_err!("Unknown hotel ID {hotel_id}"))?
            .rooms
            .remove_entry(&room_id)
            .ok_or_else(|| anyhow::format_err!("Unknown room ID {room_id}"))?;
        Ok(())
    }
    pub fn remove_hotel(&mut self, id: HotelId) -> anyhow::Result<()> {
        self.hotels
            .remove_entry(&id)
            .ok_or_else(|| anyhow::format_err!("Unknown hotel ID {id}"))?;
        Ok(())
    }
    pub fn rooms(&self) -> impl Iterator<Item = (HotelId, u64, &RoomData)> {
        self.hotels
            .iter()
            .flat_map(|(hi, hd)| hd.rooms.iter().map(|(ri, rd)| (*hi, *ri, rd)))
    }
    pub fn avaiable(
        &self,
        city: &str,
        category: Category,
        time: Range<&Date>,
    ) -> impl Iterator<Item = (HotelId, u64, &Price)> {
        self.hotels
            .iter()
            .filter(move |(_, hd)| hd.city == city)
            .flat_map(move |(hi, hd)| {
                let time = time.clone();
                hd.rooms.iter().filter_map(move |(ri, rd)| {
                    if rd.category != category {
                        return None;
                    }
                    if rd.is_occupied(time.clone()) {
                        return None;
                    }
                    Some((*hi, *ri, &rd.price))
                })
            })
    }
    pub fn book(
        &mut self,
        hotel_id: HotelId,
        room_id: u64,
        time: Range<Date>,
        customer: CustomerId,
    ) -> anyhow::Result<BookingId> {
        let room = self
            .hotels
            .get_mut(&hotel_id)
            .ok_or_else(|| anyhow::format_err!("Unknown hotel ID {hotel_id}"))?
            .rooms
            .get_mut(&room_id)
            .ok_or_else(|| anyhow::format_err!("Unknown room number {hotel_id}"))?;
        if room.is_occupied({
            let Range { start, end } = &time;
            Range { start, end }
        }) {
            bail!("Room is already occupied during that time frame");
        }
        let id = self.cur_booking_id;
        self.cur_booking_id += 1;
        room.bookings.push(Booking { time, customer, id });
        Ok(id)
    }

    pub fn get_customer(&mut self, person: Person) -> CustomerId {
        *self.customers.entry(person).or_insert_with(|| {
            let id = self.cur_customer_id;
            self.cur_customer_id += 1;
            id
        })
    }

    pub fn cancel(&mut self, booking_id: BookingId, customer: CustomerId) -> anyhow::Result<()> {
        for hotel in self.hotels.values_mut() {
            for room in hotel.rooms.values_mut() {
                let Some(idx) = room
                    .bookings
                    .iter()
                    .position(|booking| booking.id == booking_id)
                else {
                    continue;
                };
                ensure!(
                    room.bookings[idx].customer == customer,
                    "This booking does not belong to customer {customer}"
                );
                room.bookings.swap_remove(idx);
                return Ok(());
            }
        }
        bail!("Could not find booking with id {booking_id}")
    }

    pub fn bookings(&self) -> impl Iterator<Item = &Booking> {
        self.hotels
            .values()
            .flat_map(|hotel| hotel.rooms.values().flat_map(|room| room.bookings.iter()))
    }
}
