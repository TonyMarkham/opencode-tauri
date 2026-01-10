

# Efficient Prompt Workflow

## Scrum Master Agent Session (Research)

### Init
```txt
We have now implemented session 13
```

### Followup
```txt
Looks like we have finished Phase 2, if you agree, can you please update `SESSION_PLAN_FEATURE_BASED_FINAL.md` to reflect this
```

### Plan
```txt
Please look at planning how to implement Session 14 1st to give the developer agent a solid foundation. Recognize, your role is to plan, not to code.
```

---

## Developer-Teacher Agent Session (Plan)

### Init
```txt
read NEXT_SESSION_PROMPT.md
```

### Followup
```txt
I think there was some required reading that you avoided
```

### Initial Plan
```txt
before you begin teaching me, I want you to fully think out and plan the material ahead of time and present this plan
```

### Challenge
```txt
Now that it seems like you've researched what you want to do, I'd like you to evaluate your plan and rate it based on how production grade it is.
```

### Product-Grade
```text
I want you to plan to give me something that's over 9 out of 10 on a production grade score and don't stop iterating until you do.
```

### Document
```txt
Can you please serialize that to `session_14_prod_plan.md`?
```

---

## Developer-Teacher Agent Session (Implementation)

### Implement
```txt
I'd like you to read `CRITICAL_OPERATING_CONSTRAINTS.md`  And using those guidelines help me implement `session_14_prod_plan.md`

 You won't be writing or editing any code. You'll be presenting code snippets to me, for me to implement, so that I understand what's going on incrementally.

 Please break the presentation to me up into digestible slices that won't over-tax my cognitive awareness.
```