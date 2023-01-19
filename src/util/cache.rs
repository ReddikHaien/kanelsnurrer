use std::{collections::{BTreeMap, HashMap}, fmt::Debug};

pub struct Cache<Key: CacheKey, Value: Clone>{
    map: BTreeMap<Key, Value>,
    default_value: Option<Value>
}

impl<Key: CacheKey, Value: Clone + Debug> Debug for Cache<Key, Value>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (key,value) in &self.map{
            map.entry(key, value);
        }

        if let Some(ref value) = &self.default_value{
            map.entry(&"__default__", value);
        }
        map.finish()
    }
}

impl<Key: CacheKey, Value: Clone> Cache<Key, Value>{
    pub fn new() -> Self{
        Self{
            map: BTreeMap::new(),
            default_value: None
        }
    }

    pub fn new_with_default(default: Value) -> Self{
        Self{
            map: BTreeMap::new(),
            default_value: Some(default)
        }
    }

    ///
    /// Returns the value stored at the provided Key, or the default value.
    pub fn get(&self, key: &Key) -> Option<&Value>{
        self.map.get(key)
    }

    pub fn get_recursive(&self, key: &Key) -> Option<&Value>{
        match self.map.get(key){
            Some(value) => Some(value),
            None => {
                let mut c = key.parent();
                while let Some(p) = c {
                    match self.map.get(&p){
                        Some(value) => {
                            return Some(value);
                        },
                        None => {
                            c = p.parent();
                        },
                    }
                }

                self.default_value.as_ref()
            }
        }
    }

    pub fn set_default(&mut self, value: Value) -> Option<Value>{
        self.default_value.replace(value)
    }

    pub fn set(&mut self, key: Key, value: Value) -> Option<Value>{
        self.map.insert(key, value)
    }

    pub fn get_or_initialize_with<F>(&mut self, key: Key, init: F) -> &Value
    where
        F: FnOnce () -> Value
    {
        let value = self.map.entry(key).or_insert_with(init) as &Value;
        value
    }

    pub fn get_or_initialize_with_parent(&mut self, key: &Key) -> Result<Option<&Value>,Option<&Value>>{
        assert!(self.default_value.is_some(),"Cache requires a default if it's initializing lazily");

        if self.map.contains_key(key){
            Ok(self.get(key))
        }
        else{
            let Some(inherited) = self.get_recursive(key).cloned() else { unreachable!() };
            self.map.insert(key.clone(), inherited);
            Err(self.get(key))
        }
    }
}


pub trait CacheKey: Ord + Eq + PartialEq + PartialOrd + Clone + Debug{
    fn parent(&self) -> Option<Self>;
    fn is_child_of(&self, parent: &Self) -> bool{
        match self.parent(){
            Some(actual) => parent.eq(&actual),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests{
    use std::{rc::Rc};

    use crate::util::cache::Cache;

    use super::CacheKey;


    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
    struct DummyKey(Rc<InnerDummyKey>);
    
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
    struct InnerDummyKey{
        parent: Option<DummyKey>,
        value: i32
        
    }

    impl CacheKey for DummyKey{
        fn parent(&self) -> Option<Self> {
            self.0.parent.clone()
        }
    }

    #[test]
    fn set_adds_value(){
        //ARRANGE
        let mut cache: Cache<DummyKey, &str>;
        let key = DummyKey(Rc::new(InnerDummyKey{
            parent: None,
            value: 1
        }));

        let value = "a value";

        //ACT
        cache = Cache::new();
        cache.set(key.clone(), value);

        let returned = cache.get(&key);

        //ASSERT
        assert!(cache.map.len() == 1,"There should only be one element in the cache");
        assert!(returned.eq(&Some(&value)),"Should be the same value");
    }

    #[test]
    fn get_returns_correct_values(){
            //ARRANGE
            let mut cache: Cache<DummyKey, &str>;
            let parent= DummyKey(Rc::new(InnerDummyKey{
                parent: None,
                value: 1
            }));

            let child = DummyKey(Rc::new(InnerDummyKey{
                parent: Some(parent.clone()),
                value: 2
            }));
    
            let value1 = "a value";
            let value2 = "b value";
            //ACT
            cache = Cache::new();
            cache.set(parent.clone(), value1);
            cache.set(child.clone(), value2);
    
            let returned1 = cache.get(&parent);
            let returned2 = cache.get(&child);

            //ASSERT
            assert_eq!(returned1,Some(&value1));
            assert_eq!(returned2, Some(&value2));

    }

    #[test]
    fn get_missing_entry_returns_none(){
        //ARRANGE
        let mut cache: Cache<DummyKey, &str>;
        let key = DummyKey(Rc::new(InnerDummyKey{
            parent: None,
            value: 1
        }));
        
        let default_value = "some default";

        //ACT
        cache = Cache::new_with_default(default_value);
        let returned = cache.get(&key);
    
        //ASSERT
        assert_eq!(returned,None);
    }

    #[test]
    fn get_recursive_parent_present_returns_value(){
        let mut cache: Cache<DummyKey, &str>;
        let parent = DummyKey(Rc::new(InnerDummyKey{
            parent: None,
            value: 1
        }));
        
        let child1 = DummyKey(Rc::new(InnerDummyKey{
            parent: Some(parent.clone()),
            value: 2
        }));

        let child2 = DummyKey(Rc::new(InnerDummyKey{
            parent: Some(child1.clone()),
            value: 3
        }));

        let child3 = DummyKey(Rc::new(InnerDummyKey{
            parent: Some(child2.clone()),
            value: 4
        }));
        
        let other = DummyKey(Rc::new(InnerDummyKey{
            parent: None,
            value: 5
        }));

        let value = "parent value";
        let default_value = "default value";
        let other_value = "other value";

        //ACT
        cache = Cache::new_with_default(default_value);
        cache.set(parent.clone(), value);
        cache.set(child3.clone(), other_value);

        let result1 = cache.get_recursive(&child1);
        let result2 = cache.get_recursive(&child2);
        let result3 = cache.get_recursive(&child3);
        let result4 = cache.get_recursive(&other);

        //ASSERT
        assert_eq!(result1,Some(&value));
        assert_eq!(result2,Some(&value));
        assert_eq!(result3,Some(&other_value));
        assert_eq!(result4,Some(&default_value));
    }

    #[test]
    fn get_or_initialize_with_parent_given_missing_entries_initializes_child(){
        let mut cache: Cache<DummyKey, &str>;
        let parent = DummyKey(Rc::new(InnerDummyKey{
            parent: None,
            value: 1
        }));
        
        let child1 = DummyKey(Rc::new(InnerDummyKey{
            parent: Some(parent.clone()),
            value: 2
        }));

        let child2 = DummyKey(Rc::new(InnerDummyKey{
            parent: Some(child1.clone()),
            value: 3
        }));

        let child3 = DummyKey(Rc::new(InnerDummyKey{
            parent: Some(child2.clone()),
            value: 4
        }));

        let default_value = "default value";

        //ACT
        cache = Cache::new_with_default(default_value);
        
        //ASSERT
        assert_eq!(cache.map.len(),0);
        {
            let r1 = cache.get_or_initialize_with_parent(&child3);
            assert_eq!(r1,Err(Some(&default_value))); 
        }
        assert_eq!(cache.map.len(),1);
        {
            let r2 = cache.get_or_initialize_with_parent(&child2);
            assert_eq!(r2,Err(Some(&default_value)));
        }
        assert_eq!(cache.map.len(),2);
        {
            let r1 = cache.get_or_initialize_with_parent(&child1);
            assert_eq!(r1,Err(Some(&default_value)));
        }
        assert_eq!(cache.map.len(),3);
        {
            let r1 = cache.get_or_initialize_with_parent(&parent);
            assert_eq!(r1,Err(Some(&default_value)));
        }
        assert_eq!(cache.map.len(),4);
    }

    #[test]
    fn new_creates_new_with_none_default(){
        //ARRANGE
        let cache: Cache<DummyKey, &str>;
        
        //ACT
        cache = Cache::<DummyKey, &'static str>::new();

        //ASSERT
        assert!(cache.map.len() == 0, "Map should be empty");
        assert!(cache.default_value.is_none(),"Default should be set to none")
    }

    #[test]
    fn new_with_default_creates_new_with_the_provided_default(){
        //ARRANGE
        let cache: Cache<DummyKey, &str>;
        let default_value = "this is the default";
        
        //ACT
        cache = Cache::<DummyKey, &'static str>::new_with_default(default_value);

        //ASSERT
        assert!(cache.map.len() == 0, "Map should be empty");
        assert_eq!(cache.default_value,Some("this is the default"),"Default should be set to the provided value");
    }


}