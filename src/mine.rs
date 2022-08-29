use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

struct Node {
    content: i32,
    prev: Option<Weak<RefCell<Node>>>,
    next: Option<Rc<RefCell<Node>>>,
}

struct NaiveLruCache {
    capacity: usize,
    before_newest: Rc<RefCell<Node>>,
    after_oldest: Rc<RefCell<Node>>,
    item_map: HashMap<i32, Rc<RefCell<Node>>>,
}

fn leave_list(x: &Rc<RefCell<Node>>) {
    let mut x2 = x.as_ref().borrow_mut();
    let prev_node = x2.prev.take().unwrap().upgrade().unwrap();
    let next_node = x2.next.take().unwrap();
    prev_node.as_ref().borrow_mut().next = Some(Rc::clone(&next_node));
    next_node.as_ref().borrow_mut().prev = Some(Rc::downgrade(&prev_node));
}

fn insert_x_after_y(x: &Rc<RefCell<Node>>, y: &Rc<RefCell<Node>>) {
    let y2 = y
        .as_ref()
        .borrow_mut()
        .next
        .take()
        .unwrap();

    y.as_ref().borrow_mut().next = Some(Rc::clone(x));
    y2.as_ref().borrow_mut().prev = Some(Rc::downgrade(x));
    x.as_ref().borrow_mut().next = Some(y2);
    x.as_ref().borrow_mut().prev = Some(Rc::downgrade(y));
}

impl NaiveLruCache {
    pub fn new(capacity :usize) -> Self {
        let fake_head = Rc::new(RefCell::new( Node{
            content: 0,
            prev: None,
            next: None
        }));
        let fake_tail = Rc::new(RefCell::new(Node{
            content: 1,
            prev: None,
            next: None
        }));

        fake_head.as_ref().borrow_mut().next = Some(Rc::clone(&fake_tail));
        fake_tail.as_ref().borrow_mut().prev = Some(Rc::downgrade(&fake_head));

        NaiveLruCache {
            capacity,
            before_newest: fake_head,
            after_oldest: fake_tail,
            item_map: HashMap::new(),
        }
    }

    pub fn put(&mut self, item: i32) -> bool {
        if let Some(node) = self.item_map.get(&item) {
            leave_list(node);
            insert_x_after_y(node, &self.before_newest);
            true
        } else {
            let new_node = Rc::new(RefCell::new(Node{
                content: item,
                prev: None,
                next: None
            }));

            insert_x_after_y(&new_node, &self.before_newest);
            self.item_map.insert(item, Rc::clone(&new_node));

            if self.item_map.len() > self.capacity {
                let oldest = self.after_oldest.as_ref().borrow_mut().prev.take().unwrap().upgrade().unwrap();
                leave_list(&oldest);
                self.item_map.remove(&oldest.as_ref().borrow().content);
            }

            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::mine::NaiveLruCache;

    #[test]
    fn basics() {
        let mut cache = NaiveLruCache::new(2);
        assert_eq!(false, cache.put(999));
        assert_eq!(false, cache.put(888));
        assert_eq!(true, cache.put(999));
        assert_eq!(false, cache.put(777));
        assert_eq!(false, cache.put(888));
        assert_eq!(true, cache.put(777));
        assert_eq!(true, cache.put(888));
    }
}