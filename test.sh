#!/bin/bash
# Macかどうかを判定し、環境変数設定.
if [ "$(uname)" == 'Darwin' ]; then
    export TARGET="mac"
fi

# テスト用関数
# 第一引数：テストする値
# 第二引数：期待値
# 第三引数：テスト番号
test() {
    # アセンブリを出力し、gccでcompile
    prog=./target/debug/rcc
    exe=test
    echo "$1" | $prog > ${exe}.s
    gcc ${exe}.s -o $exe
    ./${exe}
    ret=$?
    [ ! $ret -eq "$2" ] && echo "[TestNo.$3] failed (actual $ret but expected $2)"
    rm ${exe}.s ${exe}
}

# ビルドを実行してからテスト
cargo b

test "1+2" 3 1
test "10+20" 30 2
test "100+101" 201 3
test "20-1" 19 4
test "20-10" 10 5
test "20-19" 1 6
test "1+2+3" 6 7
test "1+2-3" 0 8
test "100-2-3" 95 9
test "2*4" 8 10
test "3*4" 12 11
test "5*2*3" 30 12
test "5*2*3-10" 20 13
test "5+2*3" 11 14
test "2*3+3*4" 18 15
test "(12+16)" 28 16
test "(29-16)" 13 17
test "(12+16)+3" 31 18
test "3+(12+16)" 31 19
test "(10+4)*10" 140 20
test "10*(10+4)" 140 21
test "10/5" 2 22
test "20/5/2" 2 23
test "20/3" 6 24
test "2+20/3" 8 25
test "20/3+3" 9 26
test "20%3" 2 27
test "10+20%3" 12 28
test "2==2" 1 29
test "2+2==2*2" 1 30
test "20/10==2" 1 31
test "1==2" 0 32
test "2!=2" 0 33
test "2+2!=2*2" 0 34
test "20/10!=2" 0 35
test "1!=2" 1 36
test "1>2" 0 37
test "1<2" 1 38
test "1+3-1>2*4" 0 39
test "1*3+20>4*2/2" 1 40
test "1>=2" 0 41
test "1>=1" 1 42
test "2>=1" 1 43
test "1<=2" 1 44
test "2<=2" 1 45
test "2<=3" 1 46
test "1+3-1>=2*4" 0 47
test "1+3-1+5>=2*4" 1 48
test "1*3+20>=4*2/2" 1 49
test "1*3+20>=23" 1 50
test "1+3-1<=2*4" 1 51
test "1+3-1+5<=2*4" 1 52
test "1*3+20<=4*2/2" 0 53
test "1*3+20<=23" 1 54
test "3*(2+2) >= 4+(3*1)" 1 55
test "1&&1" 1 56
test "0&&1" 0 57
test "(1 + 1) && (2 * 1)" 1 58
test "1 == 1 && 2 < 1" 0 59
test "4 / 2 == 0 + 2 && 2 > 1" 1 60
test "1||1" 1 61
test "0||0" 0 62
test "(1 + 1) || (2 * 1)" 1 63
test "1 != 1 || 2 < 1" 0 64
test "4 / 2 == 0 + 2 || 2 < 1" 1 65
test "(1 == 0 && 1) && (2 < 1 || 0)" 0 66
test "2 ? 1 : 3" 1 67
test "2 > 1 ? 1 : 3" 1 68
test "2 < 1 ? 1 : 3" 3 69
test "2 > 1 ? (2 ? 10 : 100) : 3" 10 70
test "2 == 1 ? (2 == 2 ? 9 : 99) : (0 ? 10 : 100)" 100 71
#test 256 256 # 255までしか数値を扱うことができない

