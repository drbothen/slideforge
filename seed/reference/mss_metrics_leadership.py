"""MSS Metrics Leadership Deck.

Audience: Internal MSS leadership (Mattei + MSS leadership team).
Tone: Direct, opinionated, definition register.

Purpose: Define the metrics MSS should monitor to determine DPG success.
Proposes a North Star, executive health dimensions, tier-1 business metrics,
team-level rollups, and an executive dashboard. Answers the planner task:
"What are the metrics we should be monitoring for DPG to determine success?"

Usage:
    uv run python scripts/build-incident-brief.py scripts/incident_data/mss_metrics_leadership.py
"""

from pathlib import Path
from pptx.util import Inches

METADATA = {
    "incident_id": "MSS-METRICS-2026",
    "title": "MSSP Metrics That Matter",
    "date": "2026-05-22",
    "author": "Joshua Magady",
    "output": Path(__file__).parent.parent.parent / "MSS-Metrics-Leadership-Brief.pptx",
}


SLIDES = [
    # ── 1. Title ────────────────────────────────────────────────────
    {
        "type": "title",
        "color": "blue",
        "title": "MSSP Metrics That Matter",
        "subtitle": "Defining what we measure for DPG to determine success  |  MSS Leadership  |  May 2026",
        "talk_track": (
            "What we measure determines what we optimize. The question on the table is what "
            "metrics we should be monitoring for DPG to determine success. This deck answers "
            "that. A North Star that ties recurring revenue, margin, and delivery health "
            "together. Four executive dimensions that roll up into it. Team-level ownership "
            "so every group knows what they're accountable for. The result is a metric system "
            "that's defensible, hard to game, and connected directly to whether the business "
            "is actually healthy."
        ),
    },

    # ── 2. The Five Questions ───────────────────────────────────────
    {
        "type": "numbered_actions",
        "title": "An MSSP Metric System Has to Answer Five Things",
        "columns": 1,
        "actions": [
            {"text": "Can we sell it?", "category": "Commercial", "color": "blue"},
            {"text": "Can we onboard it?", "category": "Delivery", "color": "teal"},
            {"text": "Can we detect accurately?", "category": "Detection", "color": "purple"},
            {"text": "Can we operate it within SLA?", "category": "Response", "color": "orange"},
            {"text": "Can we do all of this profitably and retain the client?", "category": "Business", "color": "green"},
        ],
        "takeaway": "If our metrics can't answer all five, we're measuring the wrong things.",
        "talk_track": (
            "Five questions an MSSP metric system has to answer. "
            "Can we sell it. Can we onboard it. Can we detect accurately. "
            "Can we operate it within SLA. And can we do all of that profitably and retain the client. "
            "Sales, deployment, detection, SOC, and finance. Each is a distinct function with its own "
            "working metrics, but they all roll up to the same business question: are we running a "
            "healthy MSSP, or just a busy one. "
            "Today our metric system answers question four well. The other four are partial or "
            "implicit. That's the gap. "
            "If our metrics can't answer all five, we're measuring the wrong things."
        ),
    },

    # ── 3. The Obvious North Star Doesn't Work ─────────────────────
    {
        "type": "highlight",
        "title": "The Obvious North Star Doesn't Work",
        "highlight": (
            "The instinct for a security service is to measure downtime. "
            "It's a powerful outcome, but it's a bad business metric. "
            "Many clients will never have a downtime event, and downtime depends on "
            "patching, architecture, and BC decisions we don't control."
        ),
        "supporting": [
            "Most clients won't have a measurable downtime event in any given year",
            "Downtime is driven by client patching, architecture, and BC decisions outside MSSP control",
            "A clean year doesn't prove the service worked: it could mean we got lucky",
            "Bad incentive: celebrating 'no downtime' even when detection, response, or coverage were weak",
            "Doesn't tie to revenue retention, margin, or service scalability",
        ],
        "takeaway": "A North Star can't measure what didn't happen. It has to measure what we did.",
        "talk_track": (
            "Before we land on a North Star, let's rule out the obvious candidate. For a "
            "security service, the instinct is to measure downtime: we exist so clients don't "
            "get hurt. The instinct is right. But as a measurable business metric, downtime "
            "fails in three ways. Most clients will never have downtime to measure. The "
            "downtime that does happen is often driven by things outside our control. And we "
            "could be doing genuinely bad SOC work and still get a clean year by luck. "
            "We need a North Star that's measurable every quarter, defensible to leadership, "
            "and hard to game. "
            "A North Star can't measure what didn't happen. It has to measure what we did."
        ),
    },

    # ── 4. Section Divider: The North Star ──────────────────────────
    {
        "type": "title",
        "color": "blue",
        "title": "The North Star",
        "subtitle": "What we should be measuring at the top",
        "talk_track": (
            "Now let's look at what the North Star actually is, the math behind it, and what "
            "feeds into it."
        ),
    },

    # ── 5. Recommended North Star ──────────────────────────────────
    {
        "type": "highlight",
        "title": "Recommended North Star",
        "highlight": (
            "Retained Protected Revenue at Target Margin: "
            "the recurring revenue we retain because we consistently deliver "
            "protection outcomes within SLA, within scope, and at or above target margin."
        ),
        "supporting": [
            "Business-health metric, not a SOC metric: ties revenue, delivery, and margin together",
            "Measurable every quarter from data we already collect",
            "Won't celebrate revenue that's unprofitable, unscalable, or operationally unhealthy",
            "Hard to game: requires retention, margin, AND delivery to all be true at once",
            "Connects directly to the contracts we sign and the renewals we close",
        ],
        "talk_track": (
            "This is the proposal. Retained Protected Revenue at Target Margin. We keep "
            "recurring revenue, we keep it at the margin we said we'd keep it at, and we keep "
            "it because we're actually delivering. Three things, multiplicatively. If any one "
            "goes to zero, the metric goes to zero. That's the discipline we want. "
            "Five things make this defensible. It's a business-health metric, not a SOC metric: "
            "it ties revenue, delivery, and margin together. It's measurable every quarter from "
            "data we already collect. It won't celebrate revenue that's unprofitable, unscalable, "
            "or operationally unhealthy. It's hard to game because retention, margin, and "
            "delivery all have to be true at once. And it connects directly to the contracts we "
            "sign and the renewals we close."
        ),
    },

    # ── 6. North Star Formula ──────────────────────────────────────
    {
        "type": "formula",
        "title": "North Star Formula",
        "result": "Retained Protected Revenue at Target Margin",
        "operator": "×",
        "accent": "orange",
        "terms": [
            {
                "name": "ARR",
                "definition": "Annual recurring revenue from retained MSS clients",
                "color": "blue",
            },
            {
                "name": "Gross Margin %",
                "definition": "Profitability of the service after delivery cost",
                "color": "teal",
            },
            {
                "name": "Service Health Score",
                "definition": "Composite of SLA, detection, onboarding, response, satisfaction, platform",
                "color": "purple",
            },
        ],
        "takeaway": "Revenue alone isn't health. Margin and delivery health both have to hold.",
        "talk_track": (
            "Three multiplicands. ARR is the revenue we kept. Gross Margin is whether we kept "
            "it profitably. Service Health Score is whether we earned the right to keep it. "
            "Multiplying means any one going weak drags the whole number down. We can't "
            "celebrate an ARR number while the service is on fire. "
            "Revenue alone isn't health. Margin and delivery health both have to hold."
        ),
    },

    # ── 7. Service Health Score Composition ────────────────────────
    {
        "type": "weighted_composite",
        "title": "Service Health Score: What Makes Up 100%",
        "composite_label": "Service Health Score Components",
        "components": [
            {"name": "SLA Performance", "weight": 25, "color": "blue"},
            {"name": "Detection Quality", "weight": 20, "color": "teal"},
            {"name": "Onboarding Health", "weight": 20, "color": "purple"},
            {"name": "Escalation Quality", "weight": 15, "color": "orange"},
            {"name": "Satisfaction / Renewal Risk", "weight": 10, "color": "green"},
            {"name": "Platform Reliability", "weight": 10, "color": "red"},
        ],
        "context": (
            "Six components. SLA and detection lead because they're the contract we sign. "
            "Onboarding gets equal weight because bad onboarding creates bad SOC performance later."
        ),
        "takeaway": "A healthy service isn't one number. It's six, weighted by contract impact.",
        "talk_track": (
            "Six components. SLA Performance at 25% because that's the contract. Detection "
            "Quality at 20% because if we're noisy or missing things, the contract doesn't "
            "matter. Onboarding Health at 20%, the same weight as detection, because bad "
            "deployments show up as SOC pain three months later. Escalation Quality at 15% "
            "captures whether handoffs to clients and engineering are tight. Satisfaction "
            "and Platform Reliability at 10% each round it out. Weights are debatable. The "
            "principle isn't. "
            "A healthy service isn't one number. It's six, weighted by contract impact."
        ),
    },

    # ── 8. Section Divider: Executive Dimensions ────────────────────
    {
        "type": "title",
        "color": "purple",
        "title": "Executive Health Dimensions",
        "subtitle": "Four rollups beneath the North Star",
        "talk_track": (
            "Below the North Star sit four executive health dimensions. Each one answers a "
            "different question about whether the business is healthy. Here's how they break down."
        ),
    },

    # ── 9. Four Health Dimensions ──────────────────────────────────
    {
        "type": "enhanced_table",
        "title": "The Four Executive Health Dimensions",
        "events": [
            {
                "date": "Growth",
                "target": "Growth Health",
                "impact": "Are we creating and expanding recurring revenue?",
                "color": "blue",
            },
            {
                "date": "Retention",
                "target": "Retention Health",
                "impact": "Are clients staying, expanding, and trusting the service?",
                "color": "green",
            },
            {
                "date": "Delivery",
                "target": "Delivery Health",
                "impact": "Are we delivering the promised security outcomes?",
                "color": "teal",
            },
            {
                "date": "Scale",
                "target": "Scalability Health",
                "impact": "Can we grow without breaking the team or margin?",
                "color": "purple",
            },
        ],
        "columns": ["Theme", "Dimension", "What It Answers"],
        "col_widths": [Inches(1.5), Inches(3.2), Inches(7.1)],
        "takeaway": "All four have to be healthy. One sick dimension threatens the North Star.",
        "talk_track": (
            "Four dimensions. Growth is whether we're winning the right work. Retention is "
            "whether clients see enough value to stay and expand. Delivery is whether we're "
            "keeping our promises. Scalability is whether we can do all that without burning "
            "out the team or eroding margin. The North Star multiplies these together. "
            "All four have to be healthy. One sick dimension threatens the North Star."
        ),
    },

    # ── 10. Executive Metric Tree ──────────────────────────────────
    {
        "type": "metric_tree",
        "title": "Executive Metric Tree",
        "root": {
            "label": "Retained Protected Revenue at Target Margin",
            "color": "blue",
        },
        "categories": [
            {
                "header": "Growth Health",
                "color": "blue",
                "items": [
                    "ARR",
                    "MRR",
                    "Qualified Pipeline",
                    "Win Rate",
                    "Avg Deal Size",
                    "Profitable Bookings",
                ],
            },
            {
                "header": "Retention Health",
                "color": "green",
                "items": [
                    "Logo Retention",
                    "Net Revenue Retention",
                    "Revenue Churn",
                    "Client Satisfaction",
                    "Renewal Risk Index",
                    "Service Health by Client",
                ],
            },
            {
                "header": "Delivery Health",
                "color": "teal",
                "items": [
                    "SLA Attainment",
                    "Triage Accuracy",
                    "Escalation Quality",
                    "Detection Coverage",
                    "Detection Fidelity",
                    "Client Notifications",
                ],
            },
            {
                "header": "Scalability Health",
                "color": "purple",
                "items": [
                    "Gross Margin",
                    "Cost to Serve",
                    "Engineering O&M Load",
                    "Automation Coverage",
                    "Platform Reliability",
                    "Manual Toil Eliminated",
                ],
            },
        ],
        "footnote": "Each leaf rolls up to one dimension. Each dimension rolls up to the North Star.",
        "takeaway": "Four dimensions. One North Star. No single metric carries the business.",
        "talk_track": (
            "Here's the tree. North Star at the top. Four dimensions beneath. Six representative "
            "metrics under each dimension. Everything ladders up. When leadership reviews the "
            "business, this is what they should see. Not 60 metrics in a dashboard. Four "
            "dimensions and a North Star, with the option to drill down. "
            "Four dimensions. One North Star. No single metric carries the business."
        ),
    },

    # ── 11. Section Divider: Tier 1 Business Metrics ───────────────
    {
        "type": "title",
        "color": "blue",
        "title": "Tier 1: Business-Level Metrics",
        "subtitle": "What leadership reviews monthly or quarterly",
        "talk_track": (
            "Tier 1 is what leadership reviews monthly. The metrics that tell us whether the "
            "business is actually healthy, not just busy. Twelve metrics in total, split across "
            "the next two slides."
        ),
    },

    # ── 12. Tier 1: Financial + Growth ─────────────────────────────
    {
        "type": "enhanced_table",
        "title": "Tier 1: Financial and Growth Metrics",
        "events": [
            {"date": "Financial", "target": "ARR", "impact": "Annual recurring base. Core MSSP health metric.", "color": "blue"},
            {"date": "Financial", "target": "MRR", "impact": "Current recurring run rate.", "color": "blue"},
            {"date": "Financial", "target": "Gross Margin by Service Line", "impact": "Whether each service is commercially sustainable.", "color": "blue"},
            {"date": "Financial", "target": "Average Revenue per Client", "impact": "Whether the client mix supports our operating model.", "color": "blue"},
            {"date": "Financial", "target": "Cost to Serve per Client", "impact": "Which clients consume disproportionate effort.", "color": "blue"},
            {"date": "Growth", "target": "Profitable Bookings (Delivery-Approved Scope)", "impact": "Sales aren't celebrated until delivery can actually run them.", "color": "teal"},
        ],
        "columns": ["Category", "Metric", "Why It Matters"],
        "col_widths": [Inches(1.5), Inches(4.5), Inches(5.8)],
        "talk_track": (
            "Tier 1 financial and growth. ARR and MRR are table stakes. Gross Margin by "
            "Service Line is where we either confirm or refute that each offering is "
            "commercially viable. Average Revenue per Client tells us whether the client "
            "mix actually supports our operating model. Cost to Serve catches the clients "
            "that look profitable on paper but eat our team's hours. Profitable Bookings, "
            "with delivery-approved scope, is what counts for sales. Bookings without "
            "delivery sign-off don't count as wins."
        ),
    },

    # ── 13. Tier 1: Retention + Delivery + Scale ───────────────────
    {
        "type": "enhanced_table",
        "title": "Tier 1: Retention, Delivery, and Scale Metrics",
        "events": [
            {"date": "Retention", "target": "Net Revenue Retention", "impact": "Whether retained clients are expanding or contracting.", "color": "green"},
            {"date": "Retention", "target": "Logo Retention", "impact": "Whether clients stay.", "color": "green"},
            {"date": "Retention", "target": "Revenue Churn Rate", "impact": "Loss of recurring revenue.", "color": "green"},
            {"date": "Retention", "target": "Renewal Risk Index", "impact": "Forward-looking view of clients at risk.", "color": "green"},
            {"date": "Delivery", "target": "Service Health Score", "impact": "Composite indicator of whether delivery is healthy.", "color": "teal"},
            {"date": "Scale", "target": "Engineering O&M Load %", "impact": "Whether engineering can scale or is buried in support.", "color": "purple"},
        ],
        "columns": ["Category", "Metric", "Why It Matters"],
        "col_widths": [Inches(1.5), Inches(4.5), Inches(5.8)],
        "takeaway": "Twelve Tier 1 metrics in total. Each one tells leadership something the others can't.",
        "talk_track": (
            "Retention, delivery, and scale. NRR tells us whether clients we kept are growing "
            "with us or shrinking. Logo Retention is whether they stay at all. Revenue Churn "
            "is what we lost in dollars. Renewal Risk Index is forward-looking: what's at "
            "risk in the next quarter. Service Health Score is the composite we just walked "
            "through. Engineering O&M Load is the one that tells us whether we can scale or "
            "whether every new client is going to require heroic engineering effort. "
            "Twelve Tier 1 metrics in total. Each one tells leadership something the others can't."
        ),
    },

    # ── 14. Section Divider: Tier 2 Team-Level Metrics ─────────────
    {
        "type": "title",
        "color": "purple",
        "title": "Tier 2: Team-Level Metrics",
        "subtitle": "Who owns what, and what they're accountable for",
        "talk_track": (
            "Below Tier 1 sit the team-level metrics. Five teams, each with one primary "
            "outcome they own. We'll walk through each team and what changes for how they're "
            "measured."
        ),
    },

    # ── 15. Sales ──────────────────────────────────────────────────
    {
        "type": "content_stat",
        "title": "Sales: Profitable Demand",
        "blocks": [
            {"header": "What we measure", "description": "Qualified Pipeline · Win Rate by Offering · Attach Rate of Recurring Services · Discount / Price Realization · Scoped-Correctly Rate · Gross Margin at Sale"},
            {"header": "The key non-vanity metric", "description": "Profitable Bookings with Delivery-Approved Scope. Bookings without delivery sign-off on scope, assumptions, and pricing aren't wins yet."},
            {"header": "What this changes", "description": "Sales stops being measured on raw bookings. They're measured on bookings the business can actually deliver profitably."},
        ],
        "stat": {
            "value": "Profitable\nBookings",
            "label": "with delivery-approved scope",
            "color": "blue",
        },
        "stat_accent": "orange",
        "talk_track": (
            "Sales metrics today reward closing. That's right as far as it goes, but it "
            "doesn't catch the deals that should never have been sold: wrong scope, wrong "
            "price, wrong assumptions about what we can actually deliver. The change here is "
            "to require delivery sign-off on scope before a booking counts. It's not a slowdown. "
            "It's a quality gate that prevents pain three months later."
        ),
    },

    # ── 16. Deployment & Integration ───────────────────────────────
    {
        "type": "content_stat",
        "title": "Deployment & Integration: Clean Operational Handoff",
        "blocks": [
            {"header": "What we measure", "description": "Onboarding Cycle Time · Deployment First-Pass Success Rate · Source Integration Success Rate · Handoff Completeness Score · Deployment Defect Rate · Rework Hours per Deployment"},
            {"header": "The key non-vanity metric", "description": "Operationally Accepted Deployments Delivered Within Scope. Installed isn't done. SOC and Ops have to accept the handoff."},
            {"header": "What this changes", "description": "D&I stops being measured on 'installed something.' They're measured on whether the service can actually operate after they hand it off."},
        ],
        "stat": {
            "value": "Operationally\nAccepted",
            "label": "deployments within scope",
            "color": "teal",
        },
        "stat_accent": "orange",
        "talk_track": (
            "D&I is the bridge between sold work and operational service. If we measure them "
            "on 'installed it,' we get installs that SOC can't run. If we measure them on "
            "'SOC and Ops accepted the handoff,' we get deployments that actually work. "
            "The Handoff Completeness Score captures whether diagrams, access models, source "
            "inventory, and runbooks are present. If any of those are missing, the deployment "
            "isn't done, regardless of whether the agent is installed."
        ),
    },

    # ── 17. Engineering ────────────────────────────────────────────
    {
        "type": "content_stat",
        "title": "Engineering: Platform Leverage",
        "blocks": [
            {"header": "What we measure", "description": "Platform Availability · Ingestion Pipeline Reliability · Automation Coverage · Manual Toil Hours Eliminated · Integration Lead Time · Change Failure Rate · Cost per GB/Event Ingested"},
            {"header": "The key non-vanity metric", "description": "Operating Leverage Created. Hours eliminated + platform reliability + integration throughput + cost efficiency. Not lines of code."},
            {"header": "What this changes", "description": "Engineering isn't measured like a generic software team. They're measured on whether the platform makes everyone else faster, cheaper, and more reliable."},
        ],
        "stat": {
            "value": "Operating\nLeverage",
            "label": "created per quarter",
            "color": "purple",
        },
        "stat_accent": "orange",
        "talk_track": (
            "Engineering health for an MSSP is not the same as engineering health for a "
            "product company. Velocity and feature shipping matter less. What matters is "
            "operating leverage: does the platform mean we can serve the next client without "
            "hiring proportional headcount. Manual toil hours eliminated. Integration lead "
            "time falling. Cost per GB dropping. Those are the signals. We can build something "
            "that ships every week but burns the same engineering hours forever. That's not "
            "leverage. That's a treadmill."
        ),
    },

    # ── 18. Threat Detection ───────────────────────────────────────
    {
        "type": "content_stat",
        "title": "Threat Detection: Actionable Coverage",
        "blocks": [
            {"header": "What we measure", "description": "Detection Coverage by Scenario · Detection Coverage by Client · High-Fidelity Detection Rate · False Positive Rate · Detection-to-Runbook Coverage · Tuning Cycle Time · MITRE / Kill Chain Coverage"},
            {"header": "The key non-vanity metric", "description": "Actionable Detection Coverage. Coverage × Fidelity × Runbook Readiness × Client Applicability. Not detection count."},
            {"header": "What this changes", "description": "Detection isn't credited for writing rules. It's credited for rules that actually trigger correctly on real client data and have a runbook the SOC can run."},
        ],
        "stat": {
            "value": "Actionable\nCoverage",
            "label": "coverage × fidelity × runbook × applicability",
            "color": "orange",
        },
        "stat_accent": "blue",
        "talk_track": (
            "Detection teams everywhere get rewarded for shipping rules. That's a vanity metric. "
            "A rule that's noisy, untriageable, or unsupported by the client's actual telemetry "
            "isn't a win. It's noise the SOC has to absorb. Actionable Coverage multiplies "
            "four things together: is the scenario covered, does it produce high-fidelity alerts, "
            "does it have a runbook, and is it applicable to the client's environment. All four "
            "have to be present. Anything less is theater."
        ),
    },

    # ── 19. SOC Analysts ───────────────────────────────────────────
    {
        "type": "content_stat",
        "title": "SOC Analysts: Accurate Response Within SLA",
        "blocks": [
            {"header": "What we measure", "description": "SLA Attainment by Severity · MTTA · MTT Triage · MTT Escalate · MTT Close · Triage Accuracy · Escalation Quality · Reopened Case Rate · Runbook Adherence · Client Notification Quality"},
            {"header": "The key non-vanity metric", "description": "Accurate Triage and Escalation Within SLA. Replaces 'thumbs up ratio' with an objective quality review score."},
            {"header": "What this changes", "description": "SOC isn't measured on alert volume (a workload signal). They're measured on whether they classify, escalate, and close cases correctly within SLA."},
        ],
        "stat": {
            "value": "Accurate\n+ Within SLA",
            "label": "triage and escalation quality",
            "color": "green",
        },
        "stat_accent": "orange",
        "talk_track": (
            "SOC is close to what we have today, but with two changes. First, alert volume "
            "isn't a performance metric. It's a workload signal. Volume tells us how much "
            "to staff, not how good we are. Second, the thumbs-up ratio is too subjective to "
            "be a primary metric. We replace it with a controlled quality review score: a "
            "sample of cases reviewed against documented criteria for triage accuracy, "
            "escalation quality, runbook adherence, and notification quality."
        ),
    },

    # ── 20. Full Metric Map ────────────────────────────────────────
    {
        "type": "metric_tree",
        "title": "The Full Metric Map",
        "root": {
            "label": "Retained Protected Revenue at Target Margin",
            "color": "blue",
        },
        "categories": [
            {
                "header": "Growth Health",
                "color": "blue",
                "items": [
                    "ARR",
                    "MRR",
                    "Qualified Pipeline",
                    "Win Rate",
                    "Avg Deal Size",
                    "Attach Rate",
                    "Profitable Bookings",
                ],
            },
            {
                "header": "Retention Health",
                "color": "green",
                "items": [
                    "Logo Retention",
                    "Net Revenue Retention",
                    "Revenue Churn",
                    "Client Satisfaction",
                    "Renewal Risk Index",
                    "Service Health by Client",
                ],
            },
            {
                "header": "Delivery Health",
                "color": "teal",
                "items": [
                    "SLA Attainment",
                    "Triage Accuracy",
                    "Escalation Quality",
                    "Detection Coverage",
                    "Detection Fidelity",
                    "Operational Readiness",
                    "Client Notifications",
                ],
            },
            {
                "header": "Scalability Health",
                "color": "purple",
                "items": [
                    "Gross Margin",
                    "Cost to Serve",
                    "Engineering O&M Load",
                    "Automation Coverage",
                    "Platform Reliability",
                    "Rework Hours",
                    "Manual Toil Eliminated",
                ],
            },
        ],
        "takeaway": "Every working metric ladders up to a dimension. Every dimension ladders up to the North Star.",
        "talk_track": (
            "The full map. We saw the executive view earlier with six metrics per dimension. "
            "This is everything we'd actually track at the working level. Seven per dimension "
            "here. The point isn't to memorize the leaves. The point is that every metric "
            "we measure has a clear path up to a business-health dimension and to the North "
            "Star. No orphan metrics. No vanity metrics. "
            "Every working metric ladders up to a dimension. Every dimension ladders up to the North Star."
        ),
    },

    # ── 21. Team Rollup Matrix ─────────────────────────────────────
    {
        "type": "enhanced_table",
        "title": "Team Rollup: Primary Outcome by Team",
        "events": [
            {"date": "Sales", "target": "Profitable Bookings with Delivery-Approved Scope", "impact": "Growth Health, Margin Health", "color": "blue"},
            {"date": "D&I", "target": "Operationally Accepted Deployments Within Scope", "impact": "Delivery Health, Retention Health", "color": "teal"},
            {"date": "Engineering", "target": "Operating Leverage Created", "impact": "Scalability Health, Gross Margin", "color": "purple"},
            {"date": "Threat Detection", "target": "Actionable Detection Coverage", "impact": "Delivery Health, Retention Health", "color": "orange"},
            {"date": "SOC Analysts", "target": "Accurate Triage and Escalation Within SLA", "impact": "Delivery Health, Client Trust", "color": "green"},
            {"date": "MSS Leadership", "target": "Retained Protected Revenue at Target Margin", "impact": "Overall Business Health", "color": "red"},
        ],
        "columns": ["Team", "Primary Outcome Metric", "Rolls Up To"],
        "col_widths": [Inches(2.0), Inches(5.5), Inches(4.3)],
        "takeaway": "Every team owns one primary outcome. Every outcome rolls up to the North Star.",
        "talk_track": (
            "This is the accountability map. Sales owns profitable demand. D&I owns clean "
            "operational handoff. Engineering owns platform leverage. Threat Detection owns "
            "actionable coverage. SOC owns accurate response within SLA. Leadership owns the "
            "North Star. Each team can see exactly which executive dimensions they feed and "
            "what their primary outcome metric is. No ambiguity. "
            "Every team owns one primary outcome. Every outcome rolls up to the North Star."
        ),
    },

    # ── 22. Five Things This Metric System Has to Do ───────────────
    {
        "type": "severity_cards",
        "title": "Five Things This Metric System Has to Do",
        "gaps": [
            {
                "header": "Keep business health separate from SOC incident handling",
                "description": "SOC handling feeds business health, but it isn't the whole business. The metric system has to measure both, distinctly.",
                "severity": "STRUCTURAL",
                "color": "blue",
            },
            {
                "header": "Use Quality Review Score, not thumbs-up ratio",
                "description": "Thumbs-up is a useful feedback signal, but too subjective to be a primary metric. A sampled, controlled review against documented criteria is defensible.",
                "severity": "QUALITY",
                "color": "teal",
            },
            {
                "header": "Put margin and cost-to-serve in Tier 1",
                "description": "A client can be happy and still be commercially unhealthy. Margin and cost-to-serve belong in leadership review, not buried in finance reporting.",
                "severity": "COMMERCIAL",
                "color": "orange",
            },
            {
                "header": "Measure onboarding and deployment in their own tier",
                "description": "MSSP health isn't just about what happens after alerts arrive. Bad onboarding creates bad SOC performance later. D&I needs its own measured tier.",
                "severity": "DELIVERY",
                "color": "purple",
            },
            {
                "header": "Measure engineering scalability, not just velocity",
                "description": "If every new client requires custom engineering or heroic effort, the business isn't scaling even if ARR is growing. Engineering O&M Load is the canary.",
                "severity": "SCALE",
                "color": "green",
            },
        ],
        "takeaway": "These five together define the metric system. Each one prevents a category of measurement failure.",
        "talk_track": (
            "Five things this metric system has to do. First, keep business health separate "
            "from SOC incident handling. They're related but not the same. Second, use a "
            "controlled Quality Review Score instead of a thumbs-up ratio. Quality has to be "
            "objective. Third, put margin and cost-to-serve in Tier 1. Happiness without "
            "profitability is a problem. Fourth, measure onboarding and deployment in their "
            "own tier, not as assumed inputs. Fifth, measure engineering scalability. Because "
            "if every client requires custom work, we don't actually have a managed service. "
            "We have a consulting practice with recurring billing. "
            "These five together define the metric system. Each one prevents a category of "
            "measurement failure."
        ),
    },

    # ── 23. Executive Dashboard ────────────────────────────────────
    {
        "type": "enhanced_table",
        "title": "The 12-Metric Executive Dashboard",
        "events": [
            {"date": "North Star", "target": "Retained Protected Revenue at Target Margin", "impact": "Composite business health", "color": "red"},
            {"date": "Financial", "target": "ARR", "impact": "Recurring base", "color": "blue"},
            {"date": "Financial", "target": "Gross Margin by Service Line", "impact": "Service-level profitability", "color": "blue"},
            {"date": "Financial", "target": "Cost to Serve per Client", "impact": "Operating drag per client", "color": "blue"},
            {"date": "Growth", "target": "Profitable Bookings (Delivery-Approved Scope)", "impact": "Quality of new revenue", "color": "teal"},
            {"date": "Onboarding", "target": "Operational Readiness Acceptance", "impact": "Clean handoffs to operations", "color": "orange"},
            {"date": "Retention", "target": "Net Revenue Retention", "impact": "Expansion vs contraction", "color": "green"},
            {"date": "Retention", "target": "Renewal Risk Index", "impact": "Forward-looking churn signal", "color": "green"},
            {"date": "Delivery", "target": "SLA Attainment by Severity", "impact": "Contract compliance", "color": "purple"},
            {"date": "Delivery", "target": "Triage Accuracy Rate", "impact": "SOC quality", "color": "purple"},
            {"date": "Detection", "target": "Actionable Detection Coverage", "impact": "Coverage × fidelity × applicability", "color": "orange"},
            {"date": "Scale", "target": "Engineering O&M Load / Manual Toil", "impact": "Whether we can grow", "color": "red"},
        ],
        "columns": ["Category", "Metric", "What It Tells Us"],
        "col_widths": [Inches(1.6), Inches(5.6), Inches(4.6)],
        "talk_track": (
            "Twelve metrics. That's the whole executive dashboard. North Star at the top. "
            "Three financial. One growth. One onboarding. Two retention. Two delivery. One "
            "detection. One scale. That gives leadership a complete view of business health "
            "without becoming a metric junk drawer. Anything beyond these twelve belongs at "
            "the team level, not the executive review."
        ),
    },

    # ── 24. Bottom Line ────────────────────────────────────────────
    {
        "type": "highlight",
        "title": "The Bottom Line",
        "highlight": (
            "We shouldn't run the MSSP on a single SOC metric. "
            "We should run it on retained recurring revenue, "
            "protection outcomes, and margin discipline. All three, together."
        ),
        "supporting": [
            "North Star: Retained Protected Revenue at Target Margin",
            "Four executive dimensions: Growth, Retention, Delivery, Scalability",
            "Team-level primary outcomes, each mapping cleanly to one or more dimensions",
            "Twelve executive dashboard metrics: complete view, no junk drawer",
            "Five things this metric system has to do, built into the design",
        ],
        "talk_track": (
            "The bottom line. We don't pick a single SOC metric as the DPG North Star. "
            "We pick a composite that ties revenue, delivery, and margin together. We give "
            "leadership four dimensions to watch and twelve metrics to review. We give every "
            "team a clear primary outcome they own. And we build five things into the metric "
            "system by design: business health stays separate from SOC, quality is reviewed "
            "objectively, margin lives in Tier 1, onboarding and deployment have their own "
            "tier, and engineering scalability is measured. That's the proposal. Questions "
            "and feedback are exactly what I'm here for."
        ),
    },

    # ── 25. End ────────────────────────────────────────────────────
    {"type": "end", "color": "blue"},
]
