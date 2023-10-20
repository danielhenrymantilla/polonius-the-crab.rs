#![allow(unused)]

use {
    ::polonius_the_crab::prelude::*,
    ::std::collections::HashMap,
};

enum Error {}

fn fallible_operation(value: &'_ i32)
  -> Result<(), Error>
{
    Ok(())
}

fn get_or_insert(
    mut map: &'_ mut HashMap<i32, i32>,
) -> Result<&'_ i32, Error>
{
    polonius!(|map| -> Result<&'polonius i32, Error> {
        if let Some(value) = map.get(&22) {
            // fallible_operation(value)?;
            polonius_try!(fallible_operation(value));
            polonius_return!(Ok(value));
        }
    });
    map.insert(22, 42);
    Ok(&map[&22])
}

fn get_or_insert_from<E : From<Error>>(
    mut map: &'_ mut HashMap<i32, i32>,
) -> Result<&'_ i32, E>
{
    polonius!(|map| -> Result<&'polonius i32, E> {
        if let Some(value) = map.get(&22) {
            // fallible_operation(value)?;
            polonius_try!(fallible_operation(value));
            polonius_return!(Ok(value));
        }
    });
    map.insert(22, 42);
    Ok(&map[&22])
}

fn fallible_operation_option(value: &'_ i32)
  -> Option<()>
{
    Some(())
}

fn get_or_insert_option (
    mut map: &'_ mut HashMap<i32, i32>,
) -> Option<&'_ i32>
{
    polonius!(|map| -> Option<&'polonius i32> {
        if let Some(value) = map.get(&22) {
            // fallible_operation(value)?;
            polonius_try!(fallible_operation_option(value));
            polonius_return!(Some(value));
        }
    });
    map.insert(22, 42);
    Some(&map[&22])
}
