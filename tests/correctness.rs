use grow_box::*;

#[derive(PartialEq, Eq, Clone, Debug)]
enum List {
    Nil,
    Cons(usize, GrowBox<List>),
}

impl List {
    pub fn map<F: Copy + Fn(usize) -> usize>(self, f: F) -> Self {
        use List::*;
        match self {
            Nil => Nil,
            Cons(val, nxt) => Cons(f(val), nxt.map(|x| x.map(f))),
        }
    }
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
fn new() {
    let lst1 = make_list(&[1, 2, 3]);
    let lst2 = make_list(&[1, 2, 3]);
    let lst3 = make_list(&[4, 5, 6]);

    assert_eq!(lst1, lst2);
    assert_ne!(lst2, lst3);
}

#[test]
fn clone() {
    let lst1 = make_list(&[1, 2, 3]);
    let lst2 = lst1.clone();
    let lst3 = make_list(&[4, 5, 6]);

    assert_eq!(lst1, lst2);
    assert_ne!(lst2, lst3);
}

#[test]
fn set() {
    let lst1 = make_list(&[1, 2, 3]);
    let mut lst2 = lst1.clone();
    let lst3 = make_list(&[4, 5, 6]);

    lst2.set((*lst3).clone());

    assert_ne!(lst1, lst2);
    assert_eq!(lst2, lst3);
}

#[test]
fn map() {
    let lst1 = make_list(&[1, 2, 3]);
    let lst2 = lst1.clone().map(|lst1| lst1.map(|x| x * 2));
    let lst3 = make_list(&[2, 4, 6]);

    assert_ne!(lst1, lst2);
    assert_eq!(lst2, lst3);
}
