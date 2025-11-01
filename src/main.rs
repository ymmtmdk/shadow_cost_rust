/// シャドーバース デッキコスト最適化シミュレーター
/// 
/// このプログラムは、シャドーバースのデッキ編成において
/// コスト効率を最適化するシミュレーションツールです。
/// 
/// # 使用方法
/// ```
/// cargo run [デッキサイズ] [最大ターン数] [ループ回数] [試行回数]
/// ```
/// 
/// # パラメータ
/// - デッキサイズ (デフォルト: 30): デッキに含まれるカードの総数
/// - 最大ターン数 (デフォルト: 10): シミュレーションする最大ターン数  
/// - ループ回数 (デフォルト: 100): 最適化の反復回数
/// - 試行回数 (デフォルト: 100): 各候補に対するシミュレーション回数

extern crate shadow_cost_rust;
use shadow_cost_rust::*;
use std::env;

/// メイン関数
/// 
/// コマンドライン引数を解析し、シミュレーションを実行する。
/// パラメータが省略された場合はデフォルト値を使用。
fn main() {
    let mut argv = env::args();
    argv.next(); // プログラム名をスキップ
    
    // コマンドライン引数の解析（デフォルト値付き）
    let deck_size: u32 = argv.next().unwrap_or("30".to_string()).parse().unwrap();
    let turn_max: u32 = argv.next().unwrap_or("10".to_string()).parse().unwrap();
    let loop_count: u32 = argv.next().unwrap_or("100".to_string()).parse().unwrap();
    let trial_count: u32 = argv.next().unwrap_or("100".to_string()).parse().unwrap();

    // ヘッダー表示
    print_header();
    
    // 実行時間測定開始
    let start_time = std::time::Instant::now();
    
    // シミュレーション実行
    shadow_cost::CostSim::new(deck_size, turn_max).search_deck(loop_count, trial_count);
    
    // 実行時間表示
    let elapsed = start_time.elapsed();
    println!("\n=== EXECUTION COMPLETED ===");
    println!("Total execution time: {:.2}s", elapsed.as_secs_f64());
    println!("Thank you for using Shadowverse Deck Cost Optimizer!");
}

/// プログラム開始時のヘッダーを表示
fn print_header() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Shadowverse Deck Cost Optimizer                ║");
    println!("║                                                              ║");
    println!("║  This tool optimizes deck composition and initial hand      ║");
    println!("║  distribution to minimize wasted mana costs in games.       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}
