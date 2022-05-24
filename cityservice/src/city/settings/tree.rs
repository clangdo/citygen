use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    NonexistantKey,
}

// Yes, this does use a little extra memory,
// but duplicating the keys is much easier
// than having a complex reference system
pub struct TreeMap<V> {
    children: HashMap<String, TreeMap<V>>,
    key: String,
    value: Option<V>,
}

impl<V> TreeMap<V> {
    pub fn new() -> Self {
        TreeMap::<V> {
            children: HashMap::new(),
            key: String::new(),
            value: None,
        }
    }
    
    fn map_at<'a, K, I>(&self, key: K) -> Result<&TreeMap<V>, Error> where
        K: IntoIterator<Item = I>,
        I: Into<&'a str>,
    {
        let mut cur = self;
        for subkey in key {
            cur = cur.children.get(subkey.into()).ok_or(Error::NonexistantKey)?;
        }

        Ok(cur)
    }

    fn map_at_mut<'a, K, I>(&mut self, key: K) -> Result<&mut TreeMap<V>, Error> where
        K: IntoIterator<Item = I>,
        I: Into<&'a str>,
    {
        let mut cur = self;

        for subkey in key {
            cur = cur.children.get_mut(subkey.into()).ok_or(Error::NonexistantKey)?;
        }

        Ok(cur)
    }

    fn map_at_or_insert<'a, K, I>(&mut self, key: K) -> &mut TreeMap<V> where
        K: IntoIterator<Item = I>,
        I: Into<&'a str>,
    {
        let mut cur = self;

        for subkey in key {
            let children = &mut cur.children;
            let subkey = subkey.into();
            if !children.contains_key(subkey) {
                let mut child_map = TreeMap::new();
                child_map.key = String::from(subkey);
                children.insert(String::from(subkey), child_map);
            }
            
            cur = children.get_mut(subkey).unwrap_or_else(
                || unreachable!("Could not find map key after inserting it!")
            );
        }

        cur
    }

    pub fn add<'a, K, I, IntoValue>(&mut self, key: K, value: IntoValue) where
        K: IntoIterator<Item = I>,
        I: Into<&'a str>,
        IntoValue: Into<V>,
    {
        let association = self.map_at_or_insert(key);

        association.value = Some(value.into());
    }

    pub fn get<'a, K, I>(&self, key: K) -> Option<&V> where
        K: IntoIterator<Item = I>,
        I: Into<&'a str>,
    {
        match self.map_at(key) {
            Ok(map) => map.value.as_ref(),
            Err(Error::NonexistantKey) => None,
        }
    }

    pub fn set<'a, K, I, IntoValue>(&mut self, key: K, value: IntoValue) -> Result<(), Error> where
        K: IntoIterator<Item = I>,
        I: Into<&'a str>,
        IntoValue: Into<V>,
    {
        let association = self.map_at_mut(key)?;
        association.value = Some(value.into());
        Ok(())
    }
}

impl<'a, V> TreeMap<V> {

}

#[cfg(test)]
mod test {

    
    use super::*;
    
    #[test]
    fn create() {
        let map = TreeMap::<String>::new();

        assert_eq!(map.children.len(), 0);
        assert_eq!(map.key, String::from(""));
        assert_eq!(map.value, None);
    }

    #[test]
    fn simple_add_length() {
        let mut map = TreeMap::<String>::new();

        map.add(vec!["key"], "Hello");
        assert_eq!(map.children.len(), 1);
    }

    #[test]
    fn simple_add_and_get() {
        let mut map = TreeMap::<String>::new();

        map.add(vec!["key"], "Hello");
        assert_eq!(map.get(vec!["key"]), Some(&String::from("Hello")));
    }

    #[test]
    fn compound_add_and_get() {
        let mut map = TreeMap::<String>::new();

        map.add(vec!["key1","key2"], "Hello");
        assert_eq!(map.get(vec!["key1", "key2"]), Some(&String::from("Hello")));
    }

    #[test]
    fn get_failure() {
        let mut map = TreeMap::<String>::new();

        assert!(map.get(vec!["key1"]).is_none());
        map.add(vec!["key1", "key2"], "Hello");
        assert!(map.get(vec!["key1", "key3"]).is_none());
    }
}
