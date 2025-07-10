"use client";

import { useState } from "react";
import styles from "./page.module.scss";

export default function GamePage() {
  const [guess, setGuess] = useState("");
  const [playerSecret, setPlayerSecret] = useState("");
  const [log, setLog] = useState<string[]>([]);
  const [gameOver, setGameOver] = useState(false); // 🎯 勝敗がついたら true に

  const handleGuess = async () => {
    if (
      guess.length !== 3 ||
      new Set(guess).size !== 3 ||
      !/^\d{3}$/.test(guess)
    ) {
      alert("3桁の重複のない数字を入力してください");
      return;
    }

    try {
      const res = await fetch("http://localhost:3001/api/guess", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ guess }),
      });

      const data = await res.json();

      setLog((prev) => [
        ...prev,
        `🧍 あなた: ${guess} → ${data.player_result}`,
        `🤖 Bot: ${data.bot_guess} → ${data.bot_result}`,
      ]);
      setGuess("");

      if (
        data.player_result.includes("3 Eat") ||
        data.bot_result.includes("勝利")
      ) {
        setGameOver(true); // 🎉 勝敗判定
      }
    } catch (err) {
      console.error("エラー:", err);
      alert("通信エラーが発生しました");
    }
  };

  const handleInit = async () => {
    if (
      playerSecret.length !== 3 ||
      new Set(playerSecret).size !== 3 ||
      !/^\d{3}$/.test(playerSecret)
    ) {
      alert("3桁の重複のない数字を入力してください");
      return;
    }

    try {
      const res = await fetch("http://localhost:3001/api/init", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ player_secret: playerSecret }),
      });

      if (!res.ok) {
        throw new Error("サーバーエラー");
      }

      setLog((prev) => [
        ...prev,
        `🔒 あなたの秘密番号を登録しました: ${playerSecret}`,
      ]);
    } catch (err) {
      console.error("エラー:", err);
      alert("通信エラーが発生しました");
    }
  };

  const handleRestart = async () => {
    await fetch("http://localhost:3001/api/restart", {
      method: "POST",
    });

    setGuess("");
    setPlayerSecret("");
    setLog(["🔁 ゲームをリセットしました。"]);
    setGameOver(false);
  };

  const handleShutdown = async () => {
    await fetch("http://localhost:3001/api/shutdown", {
      method: "POST",
    });

    alert("サーバーを終了しました（開発用）");
  };

  return (
    <div className={styles.container}>
      <h1 className={styles.title}>Eat & Bite 対戦</h1>

      {/* 🔒 初期化フォーム */}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          handleInit();
        }}
        className={styles.inputGroup}
      >
        <input
          value={playerSecret}
          onChange={(e) => setPlayerSecret(e.target.value)}
          placeholder="あなたの秘密番号（例: 801）"
          className={styles.input}
        />
        <button type="submit" className={styles.button}>
          ゲーム開始！
        </button>
      </form>

      {/* 🎯 予想フォーム */}
      {!gameOver && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleGuess();
          }}
          className={styles.inputGroup}
        >
          <input
            value={guess}
            onChange={(e) => setGuess(e.target.value)}
            placeholder="予想（例: 527）"
            className={styles.input}
          />
          <button type="submit" className={styles.button}>
            予想！
          </button>
        </form>
      )}

      {/* 🔁 勝敗がついたら操作ボタン */}
      {gameOver && (
        <div className={styles.inputGroup}>
          <button onClick={handleRestart} className={styles.button}>
            🔄 再戦する
          </button>
          <button
            onClick={handleShutdown}
            className={styles.button}
            style={{ marginLeft: "1rem" }}
          >
            🛑 終了（開発用）
          </button>
        </div>
      )}

      {/* 📝 ログ */}
      <div className={styles.log}>
        {log.map((line, i) => (
          <div key={i}>{line}</div>
        ))}
      </div>
    </div>
  );
}
