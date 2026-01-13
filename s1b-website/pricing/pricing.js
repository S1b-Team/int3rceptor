// Pricing Page JavaScript
// Handle billing period switching and price updates

// Pricing data for email generation
const pricingData = {
    '1month': { label: '1 Month', professional: 59, business: 179, enterprise: 359 },
    '3months': { label: '3 Months', professional: 149, business: 449, enterprise: 899 },
    'monthly': { label: 'Monthly Subscription', professional: 49, business: 149, enterprise: 299 },
    'annual': { label: 'Annual Subscription', professional: 399, business: 1199, enterprise: 2399 }
};

let currentPeriod = 'annual'; // Default

document.addEventListener('DOMContentLoaded', () => {
    const billingSelector = document.getElementById('billing-period');

    if (billingSelector) {
        billingSelector.addEventListener('change', (e) => {
            currentPeriod = e.target.value;
            updatePricing(currentPeriod);
            updateButtons(currentPeriod);
        });
    }
});

function updatePricing(period) {
    // Get all price elements
    const priceElements = document.querySelectorAll('.price');
    const periodElements = document.querySelectorAll('.period');

    priceElements.forEach(priceEl => {
        const newPrice = priceEl.getAttribute(`data-${period}`);
        if (newPrice) {
            priceEl.textContent = '$' + newPrice;
        }
    });

    periodElements.forEach(periodEl => {
        const newPeriod = periodEl.getAttribute(`data-${period}`);
        if (newPeriod) {
            periodEl.textContent = newPeriod;
        }
    });
}

function updateButtons(period) {
    const data = pricingData[period];
    const periodLabel = data.label;

    // Update Professional button
    const professionalBtn = document.querySelector('.pricing-card.featured .btn-primary');
    if (professionalBtn) {
        const subject = `INT3RCEPTOR Professional License - ${periodLabel}`;
        const body = `Hello,%0D%0A%0D%0AI would like to purchase INT3RCEPTOR Professional for $${data.professional} (${periodLabel}).%0D%0A%0D%0APlease send me payment instructions and license details.%0D%0A%0D%0AThank you!`;
        professionalBtn.href = `mailto:livey_sh13lds1b33@keemail.me?subject=${encodeURIComponent(subject)}&body=${body}`;
    }

    // Update Business button
    const businessCards = document.querySelectorAll('.pricing-card');
    businessCards.forEach((card, index) => {
        const tierName = card.querySelector('.tier-name')?.textContent;
        const btn = card.querySelector('.btn');

        if (tierName === 'Business' && btn) {
            const subject = `INT3RCEPTOR Business License - ${periodLabel}`;
            const body = `Hello,%0D%0A%0D%0AI would like to purchase INT3RCEPTOR Business (5 users) for $${data.business} (${periodLabel}).%0D%0A%0D%0APlease send me payment instructions and license details.%0D%0A%0D%0AThank you!`;
            btn.href = `mailto:livey_sh13lds1b33@keemail.me?subject=${encodeURIComponent(subject)}&body=${body}`;
        }

        if (tierName === 'Enterprise' && btn) {
            const subject = `INT3RCEPTOR Enterprise License - ${periodLabel}`;
            const body = `Hello,%0D%0A%0D%0AI would like to discuss INT3RCEPTOR Enterprise (unlimited users) for $${data.enterprise} (${periodLabel}).%0D%0A%0D%0APlease contact me to discuss our requirements.%0D%0A%0D%0AThank you!`;
            btn.href = `mailto:livey_sh13lds1b33@keemail.me?subject=${encodeURIComponent(subject)}&body=${body}`;
        }
    });
}
