use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::{Rc, Weak};

struct Node<T> {
    content: T,
    prev: Option<Weak<RefCell<Node<T>>>>,
    next: Option<Weak<RefCell<Node<T>>>>,
}

pub struct NaiveLruCache<T> {
    capacity: usize,
    before_newest: Rc<RefCell<Node<T>>>,
    after_oldest: Rc<RefCell<Node<T>>>,
    item_map: HashMap<T, Rc<RefCell<Node<T>>>>,
}

///Remove a doubly-linked list node from the list, connect its prev and next.
fn detach<T>(node_ptr: &Rc<RefCell<Node<T>>>) {
    let mut node = node_ptr.as_ref().borrow_mut();
    let prev_node_ptr = node.prev.take();
    let next_node_ptr = node.next.take();
    if let Some(x) = prev_node_ptr.clone() {
        x.upgrade().unwrap().as_ref().borrow_mut().next = next_node_ptr.clone();
    }
    if let Some(x) = next_node_ptr {
        x.upgrade().unwrap().as_ref().borrow_mut().prev = prev_node_ptr;
    }
}

fn insert_x_after_y<T>(x: &Rc<RefCell<Node<T>>>, y: &Rc<RefCell<Node<T>>>) {
    let after_y: Rc<RefCell<Node<T>>> = y.as_ref().borrow_mut().next.clone().unwrap().upgrade().unwrap();

    y.as_ref().borrow_mut().next = Some(Rc::downgrade(x));
    after_y.as_ref().borrow_mut().prev = Some(Rc::downgrade(x));
    x.as_ref().borrow_mut().next = Some(Rc::downgrade(&after_y));
    x.as_ref().borrow_mut().prev = Some(Rc::downgrade(y));
}

impl<T:Eq+Hash+Default+Clone> NaiveLruCache<T> {
    pub fn new(capacity :usize) -> Self {
        let fake_head = Rc::new(RefCell::new( Node{
            content: T::default(),
            prev: None,
            next: None
        }));
        let fake_tail = Rc::new(RefCell::new(Node{
            content: T::default(),
            prev: None,
            next: None
        }));

        fake_head.as_ref().borrow_mut().next = Some(Rc::downgrade(&fake_tail));
        fake_tail.as_ref().borrow_mut().prev = Some(Rc::downgrade(&fake_head));

        NaiveLruCache {
            capacity,
            before_newest: fake_head,
            after_oldest: fake_tail,
            item_map: HashMap::new(),
        }
    }

    pub fn put(&mut self, item: T) -> bool {
        if let Some(node) = self.item_map.get(&item) {
            detach(node);
            insert_x_after_y(node, &self.before_newest);
            true
        } else {
            let new_node = Rc::new(RefCell::new(Node{
                content: item.clone(),
                prev: None,
                next: None
            }));

            insert_x_after_y(&new_node, &self.before_newest);
            self.item_map.insert(item.clone(), new_node);

            if self.item_map.len() > self.capacity {
                let oldest = self.after_oldest.as_ref().borrow_mut().prev.take().unwrap().upgrade().unwrap();
                detach(&oldest);
                self.item_map.remove(&oldest.as_ref().borrow().content);
            }

            false
        }
    }
}

impl<T:Display> Display for NaiveLruCache<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[");
        let mut cur_ptr = self.before_newest.borrow().next.clone().unwrap().upgrade().unwrap();
        while !Rc::ptr_eq(&cur_ptr,&self.after_oldest) {
            cur_ptr = {
                let cur_node = cur_ptr.as_ref().borrow();
                write!(f, "{},", cur_node.content);
                cur_node.next.clone().unwrap().upgrade().unwrap()
            };
        }
        writeln!(f, "]")
    }
}

#[cfg(test)]
mod test {
    use crate::mine::NaiveLruCache;

    #[test]
    fn basics() {
        let mut cache : NaiveLruCache<i32> = NaiveLruCache::new(2);
        assert_eq!(false, cache.put(999));
        println!("{}", cache);
        assert_eq!(false, cache.put(888));
        println!("{}", cache);
        assert_eq!(true, cache.put(999));
        println!("{}", cache);
        assert_eq!(false, cache.put(777));
        println!("{}", cache);
        assert_eq!(false, cache.put(888));
        println!("{}", cache);
        assert_eq!(true, cache.put(777));
        println!("{}", cache);
        assert_eq!(true, cache.put(888));
        println!("{}", cache);
    }
}