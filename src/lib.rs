/// シャドーバースのデッキ編成とコスト効率最適化を行うモジュール
/// 
/// このモジュールは遺伝的アルゴリズム風の手法を用いて、
/// 最適なデッキ構成と初期手札を探索します。
pub mod shadow_cost{
    use std::collections::btree_map::BTreeMap;
    use std::collections::BTreeSet;
    use std::cmp::Ordering;
    use std::rc::Rc;

    /// 高速なXorShift疑似乱数生成器
    /// 
    /// 標準ライブラリの乱数生成器より高速で、ゲームシミュレーションには
    /// 十分な品質を持つ。シングルスレッド環境での使用を前提とする。
    mod xor_rand{
        /// グローバルな乱数シード
        static mut SEED: u32 = 9;

        /// 0からn-1の範囲で疑似乱数を生成
        /// 
        /// # Arguments
        /// * `n` - 乱数の上限（この値は含まれない）
        /// 
        /// # Returns
        /// 0からn-1の範囲の疑似乱数
        /// 
        /// # Safety
        /// グローバルなmutableな状態を変更するためunsafeを使用
        pub fn rnd(n: u32) -> u32{
            unsafe{
                SEED ^= SEED << 13;
                SEED ^= SEED >> 17;
                SEED ^= SEED << 5;
                SEED % n
            }
        }
    }

    /// カードの最大コスト値
    pub const COST_MAX: u32 = 10;
    /// ゲーム開始時の初期手札枚数
    pub const INITIAL_HAND_COUNT: u32 = 3;
    /// 手札の最大枚数
    pub const HAND_MAX: u32 = 9;

    /// カードの集合を表す構造体
    /// 
    /// コスト別の枚数をヒストグラム形式で効率的に管理する。
    /// 各操作は可能な限り高速化されており、頻繁なアクセスに適している。
    #[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub struct Cards {
        /// コスト別のカード枚数 (インデックス0 = 1コスト)
        nums: [u32; COST_MAX as usize],
        /// カードの総枚数
        size: u32,
    }

    impl Cards {
        /// 空のカード集合を作成
        /// 
        /// # Returns
        /// すべてのコストのカード枚数が0の新しいCards構造体
        pub fn new() -> Cards{
            Cards{
                nums: [0; COST_MAX as usize],
                size: 0,
            }
        }

        /// カードの総枚数を取得
        /// 
        /// # Returns
        /// このカード集合に含まれるカードの総枚数
        pub fn size(&self) -> u32{
            self.size
        }

        /// 指定されたコストのカードを1枚追加
        /// 
        /// # Arguments
        /// * `cost` - 追加するカードのコスト (1-10)
        /// 
        /// # Panics
        /// コストが1-10の範囲外の場合にパニック
        pub fn add(&mut self, cost: u32){
            assert!(cost > 0 && cost <= COST_MAX);

            self.nums[(cost-1) as usize] += 1;
            self.size += 1
        }

        /// 指定されたコストのカードを1枚削除
        /// 
        /// # Arguments
        /// * `cost` - 削除するカードのコスト (1-10)
        /// 
        /// # Panics
        /// * コストが1-10の範囲外の場合
        /// * 指定されたコストのカードが存在しない場合
        pub fn remove(&mut self, cost: u32){
            assert!(cost > 0 && cost <= COST_MAX);
            assert!(self.nums[(cost-1) as usize] > 0);

            self.nums[(cost-1) as usize] -= 1;
            self.size -= 1
        }

        /// ランダムにカードを1枚ドローして削除
        /// 
        /// カード集合からランダムに1枚選択し、それを削除して返す。
        /// 各カードが選ばれる確率は等しい。
        /// 
        /// # Returns
        /// ドローされたカードのコスト値
        /// 
        /// # Panics
        /// カード集合が空の場合にパニック
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

        /// 指定コスト以下で最大のコストのカードを検索
        /// 
        /// 指定されたコスト以下で、実際に存在するカードの中で
        /// 最もコストの高いカードのコストを返す。
        /// 
        /// # Arguments
        /// * `cost` - 検索する最大コスト
        /// 
        /// # Returns
        /// 条件を満たすカードのコスト、存在しない場合はNone
        pub fn less_than(&self, mut cost: u32) -> Option<u32>{
            while cost > 0{
                if self.nums[(cost-1) as usize] > 0{
                    return Some(cost);
                }
                cost -= 1;
            }
            None
        }

        /// 指定されたコストで可能な限りカードをプレイ
        /// 
        /// 貪欲法により、利用可能なコストで可能な限り多くの
        /// カードをプレイする。高コストのカードを優先的に使用。
        /// 
        /// # Arguments
        /// * `cost` - 利用可能な総コスト
        /// 
        /// # Returns
        /// 使用できなかった残りコスト
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

        /// ランダムにカードを変更した新しいカード集合を生成
        /// 
        /// 指定された枚数のカードをランダムに削除し、
        /// その分だけランダムなコストのカードを追加する。
        /// 遺伝的アルゴリズムの変異操作に使用。
        /// 
        /// # Arguments
        /// * `n` - 変更するカードの枚数
        /// 
        /// # Returns
        /// 変更後の新しいカード集合
        /// 
        /// # Panics
        /// 指定された枚数がカード総数を超える場合にパニック
        pub fn random_change(&self, n: u32) -> Cards{
            assert!(self.size() >= n);

            let mut deck = self.clone();
            for _ in 0..n{
                deck.draw();
                deck.random_add(1);
            }
            deck
        }

        /// 他のカード集合とランダムにカードを交換
        /// 
        /// 指定された回数だけ、お互いのカード集合から
        /// ランダムに1枚ずつカードを交換する。
        /// 
        /// # Arguments
        /// * `other` - 交換相手のカード集合
        /// * `n` - 交換回数
        pub fn random_exchange(&mut self, other: &mut Cards, n: u32){
            for _ in 0..n{
                self.add(other.draw());
                other.add(self.draw());
            }
        }

        /// ランダムなコストのカードを指定枚数追加
        /// 
        /// 1-10コストのカードを等確率で選択し、指定枚数追加する。
        /// 
        /// # Arguments
        /// * `n` - 追加するカードの枚数
        pub fn random_add(&mut self, n: u32){
            for _ in 0..n{
                self.add(xor_rand::rnd(COST_MAX) as u32 +1);
            }
        }

        /// カード集合を分割し、一部を新しい集合として取り出す
        /// 
        /// 指定された枚数のカードをランダムに選択して削除し、
        /// それらで構成される新しいカード集合を返す。
        /// 
        /// # Arguments
        /// * `n` - 分割するカードの枚数
        /// 
        /// # Returns
        /// 分割されたカードで構成される新しいカード集合
        pub fn split(&mut self, n: u32) -> Cards{
            let mut deck = Cards::new();
            for _ in 0..n{
                deck.add(self.draw());
            }
            deck
        }

        /// デバッグ用の出力メソッド
        /// 
        /// カード集合の内容をコンソールに出力する。
        pub fn p(&self){
            let mut s = String::new();
            for i in 0..COST_MAX{
                s = format!("{},{}", s, self.nums[i as usize]);
            }
            println!("{:?}", self);
        }
    }

    /// シミュレーション結果のスコアを管理する構造体
    /// 
    /// 複数回の試行結果を累積し、平均スコアを計算する。
    /// スコアが低いほど優秀（無駄になるコストが少ない）。
    #[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
    pub struct Score{
        /// 試行回数
        count: u32,
        /// 累積損失（無駄になったコストの合計）
        loss: u64,
    }

    impl Score{
        /// 新しいスコアオブジェクトを作成
        /// 
        /// # Returns
        /// 試行回数と損失が0で初期化されたScore
        pub fn new() -> Score{
            Score{
                count: 0,
                loss: 0,
            }
        }

        /// 試行結果を追加
        /// 
        /// 指定された試行回数と損失を現在の累積値に加算する。
        /// 
        /// # Arguments
        /// * `count` - 追加する試行回数
        /// * `loss` - 追加する損失値
        pub fn add(&mut self, count: u32, loss: u32){
            self.count += count;
            self.loss += loss as u64;
        }

        /// 平均スコアを計算
        /// 
        /// # Returns
        /// 1試行あたりの平均損失値
        pub fn score(&self) -> f64{
            self.loss as f64 / self.count as f64
        }

        /// 総試行回数を取得
        /// 
        /// # Returns
        /// 累積試行回数
        pub fn count(&self) -> u32{
            self.count
        }
    }

    impl Ord for Score{
        /// スコア値による順序比較を実装
        /// 
        /// 平均スコア（loss/count）が小さいほど優秀として扱う。
        /// BTreeSetでの自動ソートに使用される。
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

    /// ゲームプレイのシミュレーションを行うモジュール
    /// 
    /// 実際のシャドーバースのゲーム進行をモデル化し、
    /// 各ターンでの最適なカードプレイを実行する。
    pub mod player{
        use super::*;
        use std::cmp;

        /// 指定されたコストで手札から最適にカードをプレイ
        /// 
        /// 貪欲法により、利用可能なコストで可能な限り多くの
        /// カードをプレイする。高コストのカードを優先的に使用。
        /// 
        /// # Arguments
        /// * `hand` - プレイヤーの手札
        /// * `cost` - 利用可能な総コスト
        /// 
        /// # Returns
        /// 使用できなかった残りコスト
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

        /// 1ゲーム分のシミュレーションを実行
        /// 
        /// 指定された初期手札とデッキでゲームを開始し、
        /// 各ターンでカードドローとプレイを行う。
        /// 
        /// # Arguments
        /// * `hand` - 初期手札
        /// * `deck` - デッキ
        /// * `initiative` - 先攻かどうか（後攻の場合は1枚多くドロー）
        /// * `turn_max` - 最大ターン数
        /// 
        /// # Returns
        /// ゲーム全体で無駄になったコストの合計
        pub fn run_game(hand: &Cards, deck: &Cards, initiative: bool, turn_max: u32) -> u32{
            let mut hd = hand.clone();
            let mut dk = deck.clone();
            
            // 初期手札をINITIAL_HAND_COUNT枚まで補充
            while hd.size() < INITIAL_HAND_COUNT{
                hd.add(dk.draw());
            }

            // 後攻の場合は追加で1枚ドロー
            if !initiative{
                hd.add(dk.draw());
            }

            let mut loss: u32 = 0;
            // 各ターンのシミュレーション
            for turn in 1..turn_max+1{
                let cost = dk.draw();
                // 手札上限まで1枚ドロー
                if hd.size() < HAND_MAX{
                    hd.add(cost);
                }
                // そのターンの利用可能コストでプレイ
                loss += play(&mut hd, cmp::min(turn,  COST_MAX) as u32) as u32;
            }

            loss
        }
    }

    /// 手札とデッキのペアでの試行結果を管理する構造体
    /// 
    /// 特定の手札・デッキ構成での複数回のゲームシミュレーション結果を
    /// 累積し、その組み合わせの評価を行う。
    #[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
    pub struct Trial{
        /// 初期手札
        hand: Rc<Cards>,
        /// デッキ構成
        deck: Rc<Cards>,
        /// 累積スコア
        score: Score,
    }

    impl Trial{
        /// 新しいTrialオブジェクトを作成
        /// 
        /// # Arguments
        /// * `hand` - 初期手札の構成
        /// * `deck` - デッキの構成
        /// 
        /// # Returns
        /// スコアが初期化された新しいTrial
        pub fn new(hand: Rc<Cards>, deck: Rc<Cards>) -> Trial{
            Trial{
                hand: hand,
                deck: deck,
                score: Score::new(),
            }
        }

        /// 指定回数のゲームシミュレーションを実行
        /// 
        /// この手札・デッキ構成で指定回数のゲームを実行し、
        /// 結果をスコアに累積する。
        /// 
        /// # Arguments
        /// * `turn_max` - 各ゲームの最大ターン数
        /// * `count` - 実行するゲーム数
        pub fn trial(&mut self, turn_max: u32, count: u32){
            let mut ls = 0;
            for _ in 0..count{
                ls += player::run_game(&*self.hand, &*self.deck, true, turn_max);
            }

            self.score.add(count, ls);
        }

        /// 平均スコアを取得
        /// 
        /// # Returns
        /// 1ゲームあたりの平均損失値
        pub fn score(&self) -> f64{
            self.score.score()
        }

        /// 手札構成の参照を取得
        /// 
        /// # Returns
        /// 手札構成への参照カウンタ
        pub fn hand(&self) -> Rc<Cards>{
            self.hand.clone()
        }

        /// デッキ構成の参照を取得
        /// 
        /// # Returns
        /// デッキ構成への参照カウンタ
        pub fn deck(&self) -> Rc<Cards>{
            self.deck.clone()
        }

        /// デバッグ用の出力メソッド
        /// 
        /// Trial全体の内容をコンソールに出力する。
        pub fn p(&self){
            println!("{:?}", self);
        }
    }

    impl Ord for Trial{
        /// スコア値による順序比較を実装
        /// 
        /// 平均スコアが小さいほど優秀として扱う。
        /// BTreeSetでの自動ソートに使用される。
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

    /// 試行結果のキャッシュ管理用型エイリアス
    type TrialCacheRc =  BTreeMap<Rc<Cards>, Trial>;
    
    /// 試行結果のキャッシュと上位候補の管理を行う構造体
    /// 
    /// 同一のデッキ構成に対する重複計算を防ぎ、
    /// 常に上位候補を効率的に管理する。
    pub struct TrialCache{
        /// デッキ構成をキーとした試行結果のキャッシュ
        cache: TrialCacheRc,
        /// スコア順にソートされた上位候補群
        top_grp: BTreeSet<Rc<Trial>>,
        /// シミュレーションの最大ターン数
        turn_max: u32,
        /// 空の手札（デッキ最適化時に使用）
        empty_hand: Rc<Cards>,
    }

    impl TrialCache{
        /// 新しいTrialCacheを作成
        /// 
        /// # Arguments
        /// * `turn_max` - シミュレーションで使用する最大ターン数
        /// 
        /// # Returns
        /// 初期化されたTrialCache
        pub fn new(turn_max: u32) -> TrialCache{
            TrialCache{
                cache: TrialCacheRc::new(),
                top_grp: BTreeSet::new(),
                turn_max: turn_max,
                empty_hand: Rc::new(Cards::new()),
            }
        }

        /// 指定されたキーで試行を実行し、結果をキャッシュに保存
        /// 
        /// 既にキャッシュに存在する場合は既存の結果に追加し、
        /// 存在しない場合は新しいTrialを作成する。
        /// 
        /// # Arguments
        /// * `key` - キャッシュのキー（通常はデッキ構成）
        /// * `hd` - 手札構成
        /// * `dk` - デッキ構成
        /// * `trial_count` - 実行する試行回数
        fn trial(&mut self, key: &Rc<Cards>, hd: &Rc<Cards>, dk: &Rc<Cards>, trial_count: u32){
            if !self.cache.contains_key(key){
                self.cache.insert(key.clone(), Trial::new(hd.clone(), dk.clone()));
            }
            let t = self.cache.get_mut(key).unwrap();
            t.trial(self.turn_max, trial_count);
            self.top_grp.insert(Rc::new(t.clone()));
        }

        /// デッキ構成の変異による試行を実行
        /// 
        /// 既存の試行結果のデッキにランダムな変更を加えて
        /// 新しい候補を生成し、評価する。
        /// 
        /// # Arguments
        /// * `trial` - 変異の元となる試行結果
        /// * `trial_count` - 実行する試行回数
        pub fn deck_trial(&mut self, trial: &Rc<Trial>, trial_count: u32){
            let hd = self.empty_hand.clone();
            let dk = Rc::new(trial.deck().random_change(3));
            self.trial(&dk, &hd, &dk, trial_count);
        }

        /// 手札・デッキ分割の変異による試行を実行
        /// 
        /// 既存の試行結果の手札とデッキ間でカードを交換し、
        /// 新しい手札・デッキ分割を評価する。
        /// 
        /// # Arguments
        /// * `trial` - 変異の元となる試行結果
        /// * `trial_count` - 実行する試行回数
        pub fn hand_trial(&mut self, trial: &Rc<Trial>, trial_count: u32){
            let mut h = (*trial.hand()).clone();
            let mut d = (*trial.deck()).clone();
            h.random_exchange(&mut d, 3);

            let hd = Rc::new(h);
            let dk = Rc::new(d);
            self.trial(&hd, &hd, &dk, trial_count);
        }

        /// 上位n個の試行結果を取得
        /// 
        /// スコア順にソートされた上位候補から指定個数を返す。
        /// 
        /// # Arguments
        /// * `n` - 取得する候補数
        /// 
        /// # Returns
        /// 上位n個の試行結果のベクタ
        fn top_group(&self, n: usize) -> Vec<Rc<Trial>>{
            self.top_grp.iter().take(n).cloned().collect()
        }
    }

    /// コスト効率シミュレーションのメイン制御構造体
    /// 
    /// デッキ構成と手札分割の2段階最適化を実行し、
    /// 最適なカード構成を探索する。
    pub struct CostSim{
        /// デッキの総カード数
        deck_size: u32,
        /// シミュレーションの最大ターン数
        turn_max: u32,
    }

    /// 各世代で保持する上位候補の数
    pub const TOP_GROUP_SIZE: usize = 8;

    impl CostSim{
        /// 新しいCostSimオブジェクトを作成
        /// 
        /// # Arguments
        /// * `deck_size` - デッキの総カード数
        /// * `turn_max` - シミュレーションの最大ターン数
        /// 
        /// # Returns
        /// 初期化されたCostSim
        pub fn new(deck_size: u32, turn_max: u32) -> CostSim{
            CostSim{
                deck_size: deck_size,
                turn_max: turn_max,
            }
        }

        /// デッキ構成の最適化を実行
        /// 
        /// 遺伝的アルゴリズム風の手法でデッキ構成を最適化し、
        /// その後手札最適化フェーズに移行する。
        /// 
        /// # Arguments
        /// * `loop_count` - 最適化の世代数
        /// * `trial_count` - 各候補の評価試行回数
        pub fn search_deck(&self, loop_count: u32, trial_count: u32){
            println!("search_deck");

            // ランダムな初期デッキを生成
            let mut deck = Cards::new();
            deck.random_add(self.deck_size);

            let mut cache = TrialCache::new(self.turn_max);

            // 初期候補を評価
            let hd = Rc::new(Cards::new());
            let dk = Rc::new(deck);
            cache.deck_trial(&Rc::new(Trial::new(hd, dk)), trial_count);
            
            // 指定世代数だけ最適化を繰り返し
            for _ in 0..loop_count{
                for trial in cache.top_group(TOP_GROUP_SIZE){
                    cache.deck_trial(&trial, trial_count);
                }
            }
            
            // 最優秀デッキを表示し、手札最適化に進む
            println!("{:?}", cache.top_group(1)[0]);
            self.search_hand(cache.top_group(1)[0].deck().clone(), loop_count, trial_count);
        }

        /// 手札・デッキ分割の最適化を実行
        /// 
        /// 最適化されたデッキ構成を前提として、
        /// 最良の初期手札分割を探索する。
        /// 
        /// # Arguments
        /// * `deck` - 最適化されたデッキ構成
        /// * `loop_count` - 最適化の世代数
        /// * `trial_count` - 各候補の評価試行回数
        pub fn search_hand(&self, deck: Rc<Cards>, loop_count: u32, trial_count: u32){
            println!("search_hand");

            let mut cache = TrialCache::new(self.turn_max);

            // デッキから初期手札を分離
            let mut d = (*deck).clone();
            let hd = Rc::new(d.split(INITIAL_HAND_COUNT));
            let dk = Rc::new(d);
            cache.hand_trial(&Rc::new(Trial::new(hd, dk)), trial_count);
            
            // 指定世代数だけ最適化を繰り返し
            for _ in 0..loop_count{
                for trial in cache.top_group(TOP_GROUP_SIZE){
                    cache.hand_trial(&trial, trial_count);
                }
            }
            
            // 最終結果を表示
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
