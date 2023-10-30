use grow_box::*;
use allocation_counter;

enum List {
    Nil,
    Cons(usize, GrowBox<List>),
}

enum ByteList {
    ByteNil,
    ByteCons(u8, GrowBox<ByteList>),
}

enum DoubledList {
    DoubledNil,
    DoubledCons(usize, usize, GrowBox<DoubledList>),
}

fn make_list(items: &[usize]) -> GrowBox<List> {
    use List::*;

    let mut lst = GrowBox::new(Nil);
    for item in items.iter().rev() {
        lst = GrowBox::new(Cons(*item, lst));
    }
    lst
}

#[test]
fn map_to_itself_does_not_allocate() {
    let mut _lst = make_list(&[1, 2, 3]);

    fn to_doubled(lst: GrowBox<List>) -> GrowBox<List> {
        use List::*;
        lst.map(|lst| match lst {
            Nil => Nil,
            Cons(val, nxt) => Cons(val * 2, to_doubled(nxt)),
        })
    }

    let actual_allocs = allocation_counter::measure(|| {
        _lst = to_doubled(_lst);
    });

    assert_eq!(actual_allocs.count_total, 0);
}

#[test]
fn map_to_smaller_size_does_not_allocate() {
    let mut _lst = make_list(&[1, 2, 3]);
    let mut _lst2 = GrowBox::new(ByteList::ByteNil);

    fn to_bytes(lst: GrowBox<List>) -> GrowBox<ByteList> {
        use ByteList::*;
        use List::*;
        lst.map(|lst| match lst {
            Nil => ByteNil,
            Cons(val, nxt) => ByteCons(val as u8, to_bytes(nxt)),
        })
    }

    let actual_allocs = allocation_counter::measure(|| {
        _lst2 = to_bytes(_lst);
    });

    assert_eq!(actual_allocs.count_total, 0);
}

#[test]
fn map_to_bigger_allocates() {
    let mut _lst = GrowBox::new(List::Nil);
    let mut _lst2 = GrowBox::new(DoubledList::DoubledNil);

    let expected_allocs = allocation_counter::measure(|| {
        _lst = make_list(&[1, 2, 3]);
    });

    fn double(lst: GrowBox<List>) -> GrowBox<DoubledList> {
        use DoubledList::*;
        use List::*;
        lst.map(|lst| match lst {
            Nil => DoubledNil,
            Cons(val, nxt) => DoubledCons(val, val * 2, double(nxt)),
        })
    }

    let actual_allocs = allocation_counter::measure(|| {
        _lst2 = double(_lst);
    });

    assert_eq!(actual_allocs.count_total, expected_allocs.count_total);
}
