import json

STACK = 100000
BLIND = 1000

CLUSTERS = [169, 2197, 2197, 2197]

nodes = []


def linear_translate(s, e, pot):
    return int((s - e * pot / (STACK / (1 + e))) * pot)


def add(k, r, action, history, s0, s1, children=[]):
    res = len(nodes)

    if r == 0:
        nodes.append(
            {
                "i": 0,
                "r": r,
                "t": k % 2,
                "a": action,
                "h": history,
                "s": [s0, s1],
                "c": children,
            }
        )
    else:
        nodes.append(
            {
                "i": 0,
                "r": r,
                "t": 1 - k % 2,
                "a": action,
                "h": history,
                "s": [s0, s1],
                "c": children,
            }
        )

    return res


def dfs(k, r, action, history, s0, s1, amount):
    history += action

    assert amount >= 0
    assert s0 <= STACK
    assert s1 <= STACK
    assert s0 > 0
    assert s1 > 0

    if action == "f":
        return add(r, k, action, history, s0, s1)

    if action == "c" and (r > 3 or (s0 == STACK and s1 == STACK)):
        return add(r, k, action, history, s0, s1)

    children = []

    if r == 0:
        # fold
        children.append(dfs(k + 1, r, "f", history, s0, s1, 0))

        # call
        x0 = (s0 + amount) if k % 2 == 0 else s0
        x1 = (s1 + amount) if k % 2 == 1 else s1

        if k == 0:
            children.append(dfs(k + 1, r, "c", history, x0, x1, 0))
        else:
            children.append(dfs(0, r + 1, "c", history, x0, x1, 0))

        # raise
        for x in [x * 2 * BLIND for x in [2, 4, 8, 16]]:
            x0 = x if k % 2 == 0 else s0
            x1 = x if k % 2 == 1 else s1

            if max(x0, x1) <= STACK // 2 and x0 - s0 + x1 - s1 - amount >= amount:
                children.append(
                    dfs(k + 1, r, "x", history, x0, x1, x0 - s0 + x1 - s1 - amount)
                )

        return add(k, r, action, history, s0, s1, children)

    else:
        # fold
        children.append(dfs(k + 1, r, "f", history, s0, s1, amount))

        # call
        if k == 0:
            children.append(dfs(k + 1, r, "c", history, s0, s1, 0))
        else:
            x0 = (s0 + amount) if k % 2 == 1 else s0
            x1 = (s1 + amount) if k % 2 == 0 else s1

            assert x0 == x1

            children.append(dfs(0, r + 1, "c", history, x0, x1, 0))

        if amount == 0:
            # bet
            pot = s0 + s1

            for s, e in [(2 / 3, 1 / 3), (4 / 3, 2 / 3)]:
                a = linear_translate(s, e, pot)

                x0 = s0 + a if k % 2 == 1 else s0
                x1 = s1 + a if k % 2 == 0 else s1

                if max(x0, x1) <= STACK // 2:
                    children.append(dfs(k + 1, r, "b", history, x0, x1, a))
        else:
            # raise
            x0 = s0 + amount * 3 if k % 2 == 1 else s0
            x1 = s1 + amount * 3 if k % 2 == 0 else s1

            if max(x0, x1) <= STACK // 2:
                children.append(dfs(k + 1, r, "x", history, x0, x1, amount * 2))

    # all-in
    if action != "a":
        x0 = STACK if k % 2 == 1 else s0
        x1 = STACK if k % 2 == 0 else s1
        children.append(dfs(k + 1, r, "a", history, x0, x1, x0 - s0 + x1 - s1 - amount))

    return add(k, r, action, history, s0, s1, children)


dfs(0, 0, "", "", BLIND, BLIND + BLIND, BLIND)

index = 0

for i in range(len(nodes)):
    if len(nodes[i]["c"]) != 0:
        nodes[i]["i"] = index

        index += CLUSTERS[nodes[i]["r"]]

print(nodes)

with open("data/abstraction/poker-tree.json", "w") as f:
    json.dump(nodes, f, indent=2)
