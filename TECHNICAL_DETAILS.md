# 技術的詳細とアルゴリズム解説

## アーキテクチャ概要

このシミュレーターは、シャドーバースのゲームメカニクスを数学的にモデル化し、確率的最適化手法を用いてデッキ構成を改善します。

## ゲームモデル

### ゲームルール実装

#### ターン進行
```rust
for turn in 1..=turn_max {
    // 1. カードドロー（手札上限まで）
    if hand.size() < HAND_MAX {
        hand.add(deck.draw());
    }
    
    // 2. 利用可能コストでカードプレイ
    let available_cost = min(turn, COST_MAX);
    unused_cost += play_optimal(&mut hand, available_cost);
}
```

#### 最適プレイ戦略
プレイヤーは貪欲法でカードを選択：
```rust
fn play_optimal(hand: &mut Cards, mut cost: u32) -> u32 {
    while cost > 0 {
        if let Some(card_cost) = hand.less_than_or_equal(cost) {
            hand.remove(card_cost);
            cost -= card_cost;
        } else {
            break; // プレイ可能なカードがない
        }
    }
    cost // 残りコスト（無駄になった分）
}
```

### 評価関数

**目的**: 無駄になるコストの期待値を最小化

```
Score = E[Σ(t=1 to T) unused_cost_t]
```

ここで：
- `T`: 最大ターン数
- `unused_cost_t`: ターンtで使用できなかったコスト
- `E[]`: 期待値（複数回のシミュレーションによる平均）

## 最適化アルゴリズム

### 2段階最適化

#### Phase 1: デッキ構成最適化
```rust
fn search_deck(&self) -> BestDeck {
    let mut population = initialize_random_decks();
    
    for generation in 0..loop_count {
        // 1. 各候補を評価
        for deck in &mut population {
            deck.score = simulate_games(deck, trial_count);
        }
        
        // 2. エリート選択
        let elites = select_top_n(population, TOP_GROUP_SIZE);
        
        // 3. 変異による新候補生成
        population = generate_mutations(elites);
    }
    
    select_best(population)
}
```

#### Phase 2: 手札最適化
最適デッキが決定後、そのデッキでの最適な初期手札分割を探索：

```rust
fn search_hand(&self, optimal_deck: Deck) -> (BestHand, BestDeck) {
    let mut population = initialize_hand_deck_splits(optimal_deck);
    
    for generation in 0..loop_count {
        // エリート選択 + カード交換による変異
        let elites = select_top_n(population, TOP_GROUP_SIZE);
        population = generate_hand_mutations(elites);
    }
    
    select_best(population)
}
```

### 変異操作

#### デッキ変異
```rust
fn mutate_deck(deck: &Cards, mutation_rate: u32) -> Cards {
    let mut new_deck = deck.clone();
    
    for _ in 0..mutation_rate {
        // ランダムなカードを削除
        let removed_cost = new_deck.draw();
        // ランダムなコストのカードを追加
        let new_cost = random_cost(1, COST_MAX);
        new_deck.add(new_cost);
    }
    
    new_deck
}
```

#### 手札-デッキ変異
```rust
fn mutate_hand_deck(hand: &mut Cards, deck: &mut Cards, exchange_count: u32) {
    for _ in 0..exchange_count {
        let from_hand = hand.draw();
        let from_deck = deck.draw();
        hand.add(from_deck);
        deck.add(from_hand);
    }
}
```

## データ構造設計

### Cards構造体
```rust
pub struct Cards {
    nums: [u32; COST_MAX as usize], // コスト別枚数
    size: u32,                      // 総枚数
}
```

**設計理由**:
- コスト別のヒストグラム表現により、O(1)での枚数確認
- 固定サイズ配列により、メモリ効率とキャッシュ効率を最適化

### TrialCache構造体
```rust
pub struct TrialCache {
    cache: BTreeMap<Rc<Cards>, Trial>,  // 結果キャッシュ
    top_grp: BTreeSet<Rc<Trial>>,       // 上位候補の自動ソート
}
```

**設計理由**:
- `BTreeMap`: 同一デッキ構成の重複計算を防止
- `BTreeSet`: スコア順の自動ソートによりtop-k選択がO(k)
- `Rc`: メモリ使用量削減とクローンコスト最適化

## パフォーマンス最適化

### カスタム乱数生成器
```rust
mod xor_rand {
    static mut SEED: u32 = 9;
    
    pub fn rnd(n: u32) -> u32 {
        unsafe {
            SEED ^= SEED << 13;
            SEED ^= SEED >> 17;
            SEED ^= SEED << 5;
            SEED % n
        }
    }
}
```

**利点**:
- 標準ライブラリより高速
- ゲームシミュレーションには十分な品質
- スレッドセーフ性は犠牲（単一スレッド前提）

### unsafe最適化
```rust
for i in 0..COST_MAX {
    unsafe {
        sum += *self.nums.get_unchecked(i as usize);
    }
    if sum > r {
        return i + 1;
    }
}
```

**適用箇所**:
- 配列境界チェックの省略
- 高頻度で呼ばれる関数のみに限定使用

## 統計的考察

### 収束性
- **デッキ最適化**: 局所最適解に陥りやすいが、ランダム変異により脱出
- **手札最適化**: より小さな探索空間のため安定して収束

### サンプルサイズ
```rust
const DEFAULT_TRIAL_COUNT: u32 = 100;  // 各候補の評価回数
const DEFAULT_LOOP_COUNT: u32 = 100;   // 最適化の世代数
```

**推奨値**:
- 高精度が必要な場合: `trial_count = 1000`
- 高速探索の場合: `trial_count = 50`

### バイアスの考慮
- 先攻/後攻の違いは現在未実装
- マリガン（手札交換）は非対応
- 相手の行動は考慮せず、純粋なコスト効率のみ

## 拡張可能性

### 実装可能な改善
1. **マルチオブジェクティブ最適化**: コスト効率 + カードパワー
2. **先攻/後攻別最適化**: 異なる戦略の併用
3. **マリガン戦略**: 初期手札の選択的交換
4. **相手モデル**: 対戦相手の行動予測

### カードゲーム汎用化
基本的なフレームワークは他のTCGにも適用可能：
- Magic: The Gathering
- Hearthstone
- その他のコスト制カードゲーム