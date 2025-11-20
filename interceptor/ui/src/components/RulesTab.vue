<template>
    <div class="rules-tab">
        <div class="rules-list">
            <h3>Active Rules</h3>
            <div v-if="rules.length === 0" class="empty-state">
                No rules defined. Add one below.
            </div>
            <ul v-else>
                <li v-for="rule in rules" :key="rule.id" class="rule-item">
                    <div class="rule-summary">
                        <span
                            class="rule-type"
                            :class="rule.rule_type.toLowerCase()"
                            >{{ rule.rule_type }}</span
                        >
                        <span class="rule-desc">
                            If
                            <strong>{{
                                getConditionDesc(rule.condition)
                            }}</strong>
                            then
                            <strong>{{ getActionDesc(rule.action) }}</strong>
                        </span>
                    </div>
                    <div class="rule-actions">
                        <!-- Toggle active state could go here -->
                    </div>
                </li>
            </ul>
            <button
                v-if="rules.length > 0"
                @click="clearAll"
                class="danger-btn"
            >
                Clear All Rules
            </button>
        </div>

        <div class="add-rule-form">
            <h3>Add New Rule</h3>

            <div class="form-group">
                <label>Rule Type</label>
                <select v-model="newRule.rule_type">
                    <option value="Request">Request</option>
                    <option value="Response">Response</option>
                </select>
            </div>

            <div class="form-group">
                <label>Condition</label>
                <select v-model="conditionType">
                    <option value="UrlContains">URL Contains</option>
                    <option value="HeaderContains">Header Contains</option>
                    <option value="BodyContains">Body Contains</option>
                </select>
            </div>

            <div v-if="conditionType === 'UrlContains'" class="form-group">
                <input
                    v-model="conditionValue1"
                    placeholder="Substring to match in URL"
                />
            </div>
            <div
                v-else-if="conditionType === 'HeaderContains'"
                class="form-group"
            >
                <input v-model="conditionValue1" placeholder="Header Name" />
                <input
                    v-model="conditionValue2"
                    placeholder="Header Value Substring"
                />
            </div>
            <div
                v-else-if="conditionType === 'BodyContains'"
                class="form-group"
            >
                <input
                    v-model="conditionValue1"
                    placeholder="Substring to match in Body"
                />
            </div>

            <div class="form-group">
                <label>Action</label>
                <select v-model="actionType">
                    <option value="ReplaceBody">Replace Body</option>
                    <option value="SetHeader">Set Header</option>
                    <option value="RemoveHeader">Remove Header</option>
                </select>
            </div>

            <div v-if="actionType === 'ReplaceBody'" class="form-group">
                <input v-model="actionValue1" placeholder="Target string" />
                <input
                    v-model="actionValue2"
                    placeholder="Replacement string"
                />
            </div>
            <div v-else-if="actionType === 'SetHeader'" class="form-group">
                <input v-model="actionValue1" placeholder="Header Name" />
                <input v-model="actionValue2" placeholder="Header Value" />
            </div>
            <div v-else-if="actionType === 'RemoveHeader'" class="form-group">
                <input v-model="actionValue1" placeholder="Header Name" />
            </div>

            <button @click="addNewRule" :disabled="!isValid">Add Rule</button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useApi } from "../composables/useApi";
import type { Rule, RuleType, MatchCondition, Action } from "../types";

const { listRules, addRule, clearRules } = useApi();
const rules = ref<Rule[]>([]);

const newRule = ref<{ rule_type: RuleType }>({
    rule_type: "Request",
});

const conditionType = ref<"UrlContains" | "HeaderContains" | "BodyContains">(
    "UrlContains"
);
const conditionValue1 = ref("");
const conditionValue2 = ref("");

const actionType = ref<"ReplaceBody" | "SetHeader" | "RemoveHeader">(
    "ReplaceBody"
);
const actionValue1 = ref("");
const actionValue2 = ref("");

const isValid = computed(() => {
    if (!conditionValue1.value) return false;
    if (
        (conditionType.value === "HeaderContains" ||
            actionType.value === "ReplaceBody" ||
            actionType.value === "SetHeader") &&
        !actionValue1.value
    )
        return false;
    return true;
});

const loadRules = async () => {
    rules.value = await listRules();
};

const clearAll = async () => {
    await clearRules();
    await loadRules();
};

const addNewRule = async () => {
    let condition: MatchCondition;
    if (conditionType.value === "UrlContains") {
        condition = { UrlContains: conditionValue1.value };
    } else if (conditionType.value === "HeaderContains") {
        condition = {
            HeaderContains: [conditionValue1.value, conditionValue2.value],
        };
    } else {
        condition = { BodyContains: conditionValue1.value };
    }

    let action: Action;
    if (actionType.value === "ReplaceBody") {
        action = { ReplaceBody: [actionValue1.value, actionValue2.value] };
    } else if (actionType.value === "SetHeader") {
        action = { SetHeader: [actionValue1.value, actionValue2.value] };
    } else {
        action = { RemoveHeader: actionValue1.value };
    }

    const rule: Rule = {
        id: crypto.randomUUID(),
        active: true,
        rule_type: newRule.value.rule_type,
        condition,
        action,
    };

    await addRule(rule);
    await loadRules();

    // Reset form
    conditionValue1.value = "";
    conditionValue2.value = "";
    actionValue1.value = "";
    actionValue2.value = "";
};

const getConditionDesc = (c: MatchCondition) => {
    if ("UrlContains" in c) return `URL contains "${c.UrlContains}"`;
    if ("HeaderContains" in c)
        return `Header "${c.HeaderContains[0]}" contains "${c.HeaderContains[1]}"`;
    if ("BodyContains" in c) return `Body contains "${c.BodyContains}"`;
    return "Unknown";
};

const getActionDesc = (a: Action) => {
    if ("ReplaceBody" in a)
        return `Replace "${a.ReplaceBody[0]}" with "${a.ReplaceBody[1]}"`;
    if ("SetHeader" in a)
        return `Set Header "${a.SetHeader[0]}" to "${a.SetHeader[1]}"`;
    if ("RemoveHeader" in a) return `Remove Header "${a.RemoveHeader}"`;
    return "Unknown";
};

onMounted(loadRules);
</script>

<style scoped>
.rules-tab {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    padding: 1rem;
    height: 100%;
}

.rules-list ul {
    list-style: none;
    padding: 0;
}

.rule-item {
    background: rgba(15, 23, 42, 0.6);
    border: 1px solid rgba(148, 163, 184, 0.2);
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    border-radius: 4px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.rule-type {
    font-size: 0.7rem;
    text-transform: uppercase;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    margin-right: 0.5rem;
    font-weight: bold;
}

.rule-type.request {
    background: rgba(56, 189, 248, 0.2);
    color: #38bdf8;
}

.rule-type.response {
    background: rgba(168, 85, 247, 0.2);
    color: #a855f7;
}

.add-rule-form {
    background: rgba(30, 41, 59, 0.5);
    padding: 1.5rem;
    border-radius: 8px;
}

.form-group {
    margin-bottom: 1rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.4rem;
    color: #94a3b8;
    font-size: 0.9rem;
}

input,
select {
    width: 100%;
    background: rgba(15, 23, 42, 0.8);
    border: 1px solid rgba(148, 163, 184, 0.2);
    padding: 0.5rem;
    border-radius: 4px;
    color: #e2e8f0;
    margin-bottom: 0.5rem;
}

button {
    background: #38bdf8;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
    margin-top: 0.5rem;
}

button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.danger-btn {
    background: #ef4444;
    color: white;
}
</style>
