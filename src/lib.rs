pub mod shadow_cost{
    use std::collections::btree_map::BTreeMap;
    use std::collections::BTreeSet;
    use std::cmp::Ordering;
    use std::rc::Rc;

    mod xor_rand{
        static mut SEED: u32 = 9;

        pub fn rnd(n: u32) -> u32{
            unsafe{
                SEED ^= SEED << 13;
                SEED ^= SEED >> 17;
                SEED ^= SEED << 5;
                SEED % n
            }
        }
    }

    pub const COST_MAX: u32 = 10;
    pub const INITIAL_HAND_COUNT: u32 = 3;
    pub const HAND_MAX: u32 = 9;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub struct Cards {
        nums: [u32; COST_MAX as usize],
        size: u32,
    }

    impl Cards {
        pub fn new() -> Cards{
            Cards{
                nums: [0; COST_MAX as usize],
                size: 0,
            }
        }

        pub fn size(&self) -> u32{
            self.size
        }

        pub fn add(&mut self, cost: u32){
            assert!(cost > 0 && cost <= COST_MAX);

            self.nums[(cost-1) as usize] += 1;
            self.size += 1
        }

        pub fn remove(&mut self, cost: u32){
            assert!(cost > 0 && cost <= COST_MAX);
            assert!(self.nums[(cost-1) as usize] > 0);

            self.nums[(cost-1) as usize] -= 1;
            self.size -= 1
        }

        pub fn draw(&mut self) -> u32{
            assert!(self.size() > 0);

            let r = xor_rand::rnd(self.size as u32) as u32;
            let mut sum = 0;
            for i in 0..COST_MAX{
                unsafe{
                    sum += *self.nums.get_unchecked(i as usize);
                }
                if sum > r{
                    self.remove(i+1);
                    return i+1;
                }
            }
            assert!(false);
            0
        }

        pub fn less_than(&self, mut cost: u32) -> Option<u32>{
            while cost > 0{
                if self.nums[(cost-1) as usize] > 0{
                    return Some(cost);
                }
                cost -= 1;
            }
            None
        }

        pub fn play(&mut self, mut cost: u32) -> u32{
            while cost > 0{
                if let Some(c) = self.less_than(cost){
                    self.remove(c);
                    cost -= c;
                }else{
                    break;
                }
            }
            cost
        }

        pub fn random_change(&self, n: u32) -> Cards{
            assert!(self.size() >= n);

            let mut deck = self.clone();
            for _ in 0..n{
                deck.draw();
                deck.random_add(1);
            }
            deck
        }

        pub fn random_exchange(&mut self, other: &mut Cards, n: u32){
            for _ in 0..n{
                self.add(other.draw());
                other.add(self.draw());
            }
        }

        pub fn random_add(&mut self, n: u32){
            for _ in 0..n{
                self.add(xor_rand::rnd(COST_MAX) as u32 +1);
            }
        }

        pub fn split(&mut self, n: u32) -> Cards{
            let mut deck = Cards::new();
            for _ in 0..n{
                deck.add(self.draw());
            }
            deck
        }

        pub fn p(&self){
            let mut s = String::new();
            for i in 0..COST_MAX{
                s = format!("{},{}", s, self.nums[i as usize]);
            }
            println!("{:?}", self);
        }
    }

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
    pub struct Score{
        count: u32,
        loss: u64,
    }

    impl Score{
        pub fn new() -> Score{
            Score{
                count: 0,
                loss: 0,
            }
        }

        pub fn add(&mut self, count: u32, loss: u32){
            self.count += count;
            self.loss += loss as u64;
        }

        pub fn score(&self) -> f64{
            self.loss as f64 / self.count as f64
        }

        pub fn count(&self) -> u32{
            self.count
        }
    }

    impl Ord for Score{
        fn cmp(&self, other: &Score) -> Ordering {
            if self.score() > other.score(){
                Ordering::Greater
            }else if self.score() < other.score(){
                Ordering::Less
            }else{
                Ordering::Equal
            }
        }
    }

    pub mod player{
        use super::*;
        use std::cmp;

        fn play(hand: &mut Cards, mut cost: u32) -> u32{
            while cost > 0{
                if let Some(c) = hand.less_than(cost){
                    hand.remove(c);
                    cost -= c;
                } else{
                    break;
                }
            }
            cost
        }

        pub fn run_game(hand: &Cards, deck: &Cards, initiative: bool, turn_max: u32) -> u32{
            let mut hd = hand.clone();
            let mut dk = deck.clone();
            while hd.size() < INITIAL_HAND_COUNT{
                hd.add(dk.draw());
            }

            if !initiative{
                hd.add(dk.draw());
            }

            let mut loss: u32 = 0;
            for turn in 1..turn_max+1{
                let cost = dk.draw();
                if hd.size() < HAND_MAX{
                    hd.add(cost);
                }
                loss += play(&mut hd, cmp::min(turn,  COST_MAX) as u32) as u32;
            }

            loss
        }
    }

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
    pub struct Trial{
        hand: Rc<Cards>,
        deck: Rc<Cards>,
        score: Score,
    }

    impl Trial{
        pub fn new(hand: Rc<Cards>, deck: Rc<Cards>) -> Trial{
            Trial{
                hand: hand,
                deck: deck,
                score: Score::new(),
            }
        }

        pub fn trial(&mut self, turn_max: u32, count: u32){
            let mut ls = 0;
            for _ in 0..count{
                ls += player::run_game(&*self.hand, &*self.deck, true, turn_max);
            }

            self.score.add(count, ls);
        }

        pub fn score(&self) -> f64{
            self.score.score()
        }

        pub fn hand(&self) -> Rc<Cards>{
            self.hand.clone()
        }

        pub fn deck(&self) -> Rc<Cards>{
            self.deck.clone()
        }

        pub fn p(&self){
            println!("{:?}", self);
        }
    }

    impl Ord for Trial{
        fn cmp(&self, other: &Trial) -> Ordering {
            if self.score() > other.score(){
                Ordering::Greater
            }else if self.score() < other.score(){
                Ordering::Less
            }else{
                Ordering::Equal
            }
        }
    }

    type TrialCacheRc =  BTreeMap<Rc<Cards>, Trial>;
    pub struct TrialCache{
        cache: TrialCacheRc,
        top_grp: BTreeSet<Rc<Trial>>,
        turn_max: u32,
        empty_hand: Rc<Cards>,
    }

    impl TrialCache{
        pub fn new(turn_max: u32) -> TrialCache{
            TrialCache{
                cache: TrialCacheRc::new(),
                top_grp: BTreeSet::new(),
                turn_max: turn_max,
                empty_hand: Rc::new(Cards::new()),
            }
        }

        fn trial(&mut self, key: &Rc<Cards>, hd: &Rc<Cards>, dk: &Rc<Cards>, trial_count: u32){
            if !self.cache.contains_key(key){
                self.cache.insert(key.clone(), Trial::new(hd.clone(), dk.clone()));
            }
            let mut t = self.cache.get_mut(key).unwrap();
            t.trial(self.turn_max, trial_count);
            self.top_grp.insert(Rc::new(t.clone()));
        }

        pub fn deck_trial(&mut self, trial: &Rc<Trial>, trial_count: u32){
            let hd = self.empty_hand.clone();
            let dk = Rc::new(trial.deck().random_change(3));
            self.trial(&dk, &hd, &dk, trial_count);
        }

        pub fn hand_trial(&mut self, trial: &Rc<Trial>, trial_count: u32){
            let mut h = (*trial.hand()).clone();
            let mut d = (*trial.deck()).clone();
            h.random_exchange(&mut d, 3);

            let hd = Rc::new(h);
            let dk = Rc::new(d);
            self.trial(&hd, &hd, &dk, trial_count);
        }

        fn top_group(&self, n: usize) -> Vec<Rc<Trial>>{
            self.top_grp.iter().take(n).cloned().collect()
        }
    }

    pub struct CostSim{
        deck_size: u32,
        turn_max: u32,
    }

    pub const TOP_GROUP_SIZE: usize = 8;

    impl CostSim{

        pub fn new(deck_size: u32, turn_max: u32) -> CostSim{
            CostSim{
                deck_size: deck_size,
                turn_max: turn_max,
            }
        }

        pub fn search_deck(&self, loop_count: u32, trial_count: u32){
            println!("search_deck");

            let mut deck = Cards::new();
            deck.random_add(self.deck_size);

            let mut cache = TrialCache::new(self.turn_max);

            let hd = Rc::new(Cards::new());
            let dk = Rc::new(deck);
            cache.deck_trial(&Rc::new(Trial::new(hd, dk)), trial_count);
            for _ in 0..loop_count{
                for trial in cache.top_group(TOP_GROUP_SIZE){
                    cache.deck_trial(&trial, trial_count);
                }
            }
            println!("{:?}", cache.top_group(1)[0]);
            self.search_hand(cache.top_group(1)[0].deck().clone(), loop_count, trial_count);
        }

        pub fn search_hand(&self, deck: Rc<Cards>, loop_count: u32, trial_count: u32){
            println!("search_hand");

            let mut cache = TrialCache::new(self.turn_max);

            let mut d = (*deck).clone();
            let hd = Rc::new(d.split(INITIAL_HAND_COUNT));
            let dk = Rc::new(d);
            cache.hand_trial(&Rc::new(Trial::new(hd, dk)), trial_count);
            for _ in 0..loop_count{
                for trial in cache.top_group(TOP_GROUP_SIZE){
                    cache.hand_trial(&trial, trial_count);
                }
            }
            println!("{:?}", cache.top_group(1)[0]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::shadow_cost::*;
    #[test]
    fn new() {
        Cards::new();
    }

    #[test]
    fn size() {
        assert_eq!(0, Cards::new().size());
    }

    #[test]
    fn add() {
        let mut deck = Cards::new();
        deck.add(2);
        assert_eq!(1, deck.size());
    }

    #[test]
    fn remove() {
        let mut deck = Cards::new();
        deck.add(3);
        deck.add(4);
        deck.remove(3);
        assert_eq!(1, deck.size());
    }

    #[test]
    fn draw() {
        let mut deck = Cards::new();
        deck.add(2);
        assert_eq!(2, deck.draw());
        assert_eq!(0, deck.size());
    }

    #[test]
    fn less_than() {
        let mut deck = Cards::new();
        deck.add(3);
        deck.add(5);
        assert_eq!(Some(3), deck.less_than(4));
        assert_eq!(None, deck.less_than(1));
    }

    #[test]
    fn play() {
        let mut deck = Cards::new();
        deck.add(3);
        deck.add(5);
        assert_eq!(1, deck.play(4));
        assert_eq!(1, deck.size());
    }

    #[test]
    fn player(){
        let mut deck = Cards::new();
        for i in 1..10{
            deck.add(i);
            deck.add(i);
        }
        let mut hand = Cards::new();
        hand.add(1);
        hand.add(2);
        hand.add(3);
        player::run_game(&hand, &deck, true, 10);
    }

    use std::rc::Rc;
    #[test]
    fn trial(){
        let mut deck = Cards::new();
        for i in 1..10{
            deck.add(i);
            deck.add(i);
        }
        let mut hand = Cards::new();
        hand.add(1);
        hand.add(2);
        hand.add(3);
        let mut trial = Trial::new(Rc::new(hand), Rc::new(deck));
        trial.trial(10, 10);
    }

    #[test]
    fn cost_sim(){
        CostSim::new(30, 10).search_deck(10, 10);
    }
}
