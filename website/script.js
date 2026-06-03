// Smooth scroll for anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
    });
});

// Animate feature cards on scroll
const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            entry.target.style.opacity = '1';
            entry.target.style.transform = 'translateY(0)';
        }
    });
}, { threshold: 0.1 });

document.querySelectorAll('.feature-card, .mw-card, .download-card, .ai-content').forEach(el => {
    el.style.opacity = '0';
    el.style.transform = 'translateY(20px)';
    el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
    observer.observe(el);
});

// Typing effect for terminal demo
const termLines = document.querySelectorAll('.term-line');
termLines.forEach((line, i) => {
    line.style.opacity = '0';
    line.style.transform = 'translateX(-10px)';
    line.style.transition = `opacity 0.4s ease ${i * 0.15}s, transform 0.4s ease ${i * 0.15}s`;
});

const termObserver = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            entry.target.querySelectorAll('.term-line').forEach(line => {
                line.style.opacity = '1';
                line.style.transform = 'translateX(0)';
            });
        }
    });
}, { threshold: 0.3 });

const termDemo = document.querySelector('.terminal-demo');
if (termDemo) termObserver.observe(termDemo);
