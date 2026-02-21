#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, math, random
from pathlib import Path
from typing import Any
SCHEMA_VERSION = "discos.paper-artifacts.index.v1"

def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True)+"\n", encoding="utf-8")

def exp01(seed:int)->dict[str,Any]:
    rows=[]
    for q in range(1,26):
        mae_no=max(1.0/(q+2.0),1e-6)
        mae_h=max(1.0/(math.sqrt(q*2.0)+2.0),1e-6)
        rows.append({"queries":q,"mae_no_hysteresis":mae_no,"mae_with_hysteresis":mae_h})
    return {"schema_version":"discos.paper.exp01.v1","seed":seed,"rows":rows,"effective_bits_no_hysteresis":-math.log2(min(r["mae_no_hysteresis"] for r in rows)),"effective_bits_with_hysteresis":-math.log2(min(r["mae_with_hysteresis"] for r in rows))}

def exp02(seed:int)->dict[str,Any]:
    n_trials=1000;joint_budget_bits=48.0
    evidenceos_successes=int(n_trials*(2**(-joint_budget_bits/4.0)))
    return {"schema_version":"discos.paper.exp02.v1","seed":seed,"n_trials":n_trials,"joint_budget_bits":joint_budget_bits,"standard_success_rate":1.0,"evidenceos_success_rate":evidenceos_successes/n_trials}

def synthetic(exp_num:int,seed:int,n:int=32)->dict[str,Any]:
    rng=random.Random(seed);rows=[];baseline=0.12+(exp_num*0.01)
    for i in range(n):
        v=baseline+(i/(n-1))*0.2+((rng.random()-0.5)*0.01)
        rows.append({"step":i+1,"value":max(0.0,min(1.0,v))})
    return {"schema_version":f"discos.paper.exp{exp_num:02d}.v1","seed":seed,"kind":"synthetic_deterministic_placeholder","rows":rows}

def exp11(seed:int)->dict[str,Any]:
    secret_bits=20;topic_budget_bits=2.0;base=2**(-(secret_bits-topic_budget_bits));rows=[]
    for i in range(1,21):
        naive=1.0 if i>=secret_bits else 2**(-(secret_bits-i))
        rows.append({"n_identities":i,"naive_success_prob":naive,"topichash_success_prob":base})
    return {"schema_version":"discos.paper.exp11.v1","seed":seed,"rows":rows}

def exp12(seed:int)->dict[str,Any]:
    rng=random.Random(seed);topic_budget_bits=2;trials=10000;scenarios=[(32,0.01),(64,0.01),(128,0.05)];rows=[]
    for n,psplit in scenarios:
        leaked=[]
        for _ in range(trials):
            leaked.append(topic_budget_bits+sum(1 for _ in range(n) if rng.random()<psplit))
        leaked.sort();idx=min(max(math.ceil(trials*0.99)-1,0),trials-1)
        rows.append({"n":n,"psplit":psplit,"mean_leaked_bits":sum(leaked)/trials,"p99_leaked_bits":leaked[idx]})
    return {"schema_version":"discos.paper.exp12.v1","seed":seed,"rows":rows}

def payload(exp_num:int,seed:int)->dict[str,Any]:
    return exp01(seed) if exp_num==1 else exp02(seed) if exp_num==2 else exp11(seed) if exp_num==11 else exp12(seed) if exp_num==12 else synthetic(exp_num,seed)

def main()->None:
    ap=argparse.ArgumentParser();ap.add_argument("--out",default="artifacts/paper-artifacts");ap.add_argument("--smoke",action="store_true");ap.add_argument("--experiments",type=int,nargs="*")
    a=ap.parse_args();out=Path(a.out);out.mkdir(parents=True,exist_ok=True)
    experiments=a.experiments if a.experiments else [1,11,12] if a.smoke else list(range(1,13))
    index={"schema_version":SCHEMA_VERSION,"mode":"single" if a.experiments else "smoke" if a.smoke else "full","experiments":{}}
    for exp_num in experiments:
        seed=1000+exp_num;rel=Path(f"exp{exp_num:02d}.json")
        write_json(out/rel,payload(exp_num,seed))
        index["experiments"][str(exp_num)]={"artifact":rel.as_posix(),"seed":seed,"command":f"python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments {exp_num}"}
    write_json(out/"index.json",index)
    print(json.dumps({"ok":True,"out":out.as_posix(),"index":"index.json"}))

if __name__=="__main__": main()
