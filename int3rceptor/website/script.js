// ═══════════════════════════════════════════════════════
// S1b Landing Page JavaScript
// ═══════════════════════════════════════════════════════

document.addEventListener('DOMContentLoaded', () => {
    // Mobile Navigation Toggle
    const navToggle = document.querySelector('.nav-toggle');
    const navMenu = document.querySelector('.nav-menu');

    if (navToggle) {
        navToggle.addEventListener('click', () => {
            navMenu.classList.toggle('active');
        });
    }

    // Smooth Scroll for Anchor Links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function(e) {
            const href = this.getAttribute('href');
            if (href === '#') return;

            e.preventDefault();
            const target = document.querySelector(href);
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Terminal Animation
    animateTerminal();

    // Intersection Observer for Fade-in Animations
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('fade-in');
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    // Observe feature cards
    document.querySelectorAll('.feature-card').forEach(card => {
        observer.observe(card);
    });

    // Counter Animation
    animateCounters();
});

// Terminal Animation
function animateTerminal() {
    const requestsCount = document.querySelector('.stat-count');
    if (!requestsCount) return;

    let count = 0;
    const target = 1247;
    const duration = 2000;
    const increment = target / (duration / 50);

    const counter = setInterval(() => {
        count += increment;
        if (count >= target) {
            count = target;
            clearInterval(counter);
            // Restart after 3 seconds
            setTimeout(() => {
                count = 0;
                animateTerminal();
            }, 3000);
        }
        requestsCount.textContent = Math.floor(count).toLocaleString();
    }, 50);
}

// Animate Stats Counters
function animateCounters() {
    const stats = document.querySelectorAll('.stat-value');

    stats.forEach(stat => {
        const target = parseFloat(stat.textContent.replace(/[^0-9.]/g, ''));
        const suffix = stat.textContent.replace(/[0-9.]/g, '');
        const isPercentage = suffix.includes('%');
        const isThousands = suffix.includes('+');

        let count = 0;
        const duration = 2000;
        const increment = target / (duration / 50);

        const counter = setInterval(() => {
            count += increment;
            if (count >= target) {
                count = target;
                clearInterval(counter);
            }

            let displayValue = isPercentage
                ? count.toFixed(1) + '%'
                : isThousands
                    ? Math.floor(count).toLocaleString() + '+'
                    : Math.floor(count);

            stat.textContent = displayValue;
        }, 50);
    });
}

// Add CSS for fade-in animation
const style = document.createElement('style');
style.textContent = `
    .feature-card {
        opacity: 0;
        transform: translateY(20px);
        transition: opacity 0.6s ease, transform 0.6s ease;
    }

    .feature-card.fade-in {
        opacity: 1;
        transform: translateY(0);
    }

    @media (prefers-reduced-motion: reduce) {
        .feature-card {
            opacity: 1;
            transform: none;
        }
    }
`;
document.head.appendChild(style);
