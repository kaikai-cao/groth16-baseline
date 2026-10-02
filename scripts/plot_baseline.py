import csv
from pathlib import Path

import matplotlib.pyplot as plt


ROOT = Path(__file__).resolve().parents[1]
CSV_PATH = ROOT / "results" / "tables" / "baseline_summary.csv"
OUTPUT_PATH = ROOT / "results" / "figures" / "prove_time_vs_constraints.png"


constraints = []
prove_time = []

with CSV_PATH.open("r", encoding="utf-8-sig", newline="") as f:
    reader = csv.DictReader(f)

    for row in reader:
        constraints.append(int(row["N"]))
        prove_time.append(float(row["Prove_Median_ms"]))


plt.figure(figsize=(8, 5))
plt.plot(constraints, prove_time, marker="o")
plt.xlabel("Constraint Number")
plt.ylabel("Prove Time (ms)")
plt.title("Groth16 Prover Scaling Baseline")
plt.grid(True, alpha=0.3)
plt.tight_layout()

OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
plt.savefig(OUTPUT_PATH, dpi=200)
plt.close()

print(f"Saved: {OUTPUT_PATH}")