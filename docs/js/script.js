// Mobile navigation toggle
document.addEventListener('DOMContentLoaded', function() {
    const hamburger = document.querySelector('.hamburger');
    const navMenu = document.querySelector('.nav-menu');

    if (hamburger && navMenu) {
        hamburger.addEventListener('click', function() {
            hamburger.classList.toggle('active');
            navMenu.classList.toggle('active');
        });

        // Close mobile menu when clicking on a link
        document.querySelectorAll('.nav-link').forEach(n => n.addEventListener('click', () => {
            hamburger.classList.remove('active');
            navMenu.classList.remove('active');
        }));
    }

    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Add active class to current navigation item
    const currentLocation = location.pathname.split('/').pop() || 'index.html';
    document.querySelectorAll('.nav-link').forEach(link => {
        if (link.getAttribute('href') === currentLocation) {
            link.classList.add('active');
        } else {
            link.classList.remove('active');
        }
    });

    // Navbar scroll effect
    let lastScrollTop = 0;
    const navbar = document.querySelector('.navbar');
    
    window.addEventListener('scroll', function() {
        let scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        
        if (scrollTop > lastScrollTop && scrollTop > 100) {
            // Scrolling down
            navbar.style.transform = 'translateY(-100%)';
        } else {
            // Scrolling up
            navbar.style.transform = 'translateY(0)';
        }
        lastScrollTop = scrollTop;
    });

    // Copy code to clipboard functionality
    document.querySelectorAll('.code-block').forEach(block => {
        block.addEventListener('click', function() {
            const code = this.querySelector('code').textContent;
            navigator.clipboard.writeText(code).then(() => {
                // Show copied notification
                const notification = document.createElement('div');
                notification.textContent = 'Copied to clipboard!';
                notification.style.cssText = `
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    background: #059669;
                    color: white;
                    padding: 12px 20px;
                    border-radius: 8px;
                    z-index: 10000;
                    animation: slideIn 0.3s ease;
                `;
                document.body.appendChild(notification);
                
                setTimeout(() => {
                    notification.remove();
                }, 2000);
            });
        });
    });

    // Add copy button to code blocks
    document.querySelectorAll('.code-block').forEach(block => {
        const copyButton = document.createElement('button');
        copyButton.innerHTML = '📋';
        copyButton.style.cssText = `
            position: absolute;
            top: 8px;
            right: 8px;
            background: rgba(255, 255, 255, 0.1);
            border: none;
            color: white;
            padding: 4px 8px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            opacity: 0;
            transition: opacity 0.3s ease;
        `;
        
        block.style.position = 'relative';
        block.appendChild(copyButton);
        
        block.addEventListener('mouseenter', () => {
            copyButton.style.opacity = '1';
        });
        
        block.addEventListener('mouseleave', () => {
            copyButton.style.opacity = '0';
        });
        
        copyButton.addEventListener('click', (e) => {
            e.stopPropagation();
            const code = block.querySelector('code').textContent;
            navigator.clipboard.writeText(code);
            copyButton.innerHTML = '✅';
            setTimeout(() => {
                copyButton.innerHTML = '📋';
            }, 1000);
        });
    });

    // Terminal typing animation for demo
    const terminalDemo = document.querySelector('.terminal-body');
    if (terminalDemo) {
        const commands = [
            { type: 'command', text: 'SELECT script.py FROM C:\\Projects\\', delay: 1000 },
            { type: 'output', text: '✓ Selected: script.py', delay: 800 },
            { type: 'command', text: 'RUN STATE', delay: 1000 },
            { type: 'output', text: '✓ Executing: python script.py', delay: 1200 },
            { type: 'command', text: 'FAV SET STATE', delay: 1000 },
            { type: 'output', text: '✓ Added to favorites', delay: 800 }
        ];

        let currentCommand = 0;
        
        function typeCommand() {
            if (currentCommand < commands.length) {
                const cmd = commands[currentCommand];
                const line = document.createElement('div');
                line.className = 'terminal-line';
                
                if (cmd.type === 'command') {
                    line.innerHTML = '<span class="prompt">DIR2@C:</span><span class="command"></span>';
                    const commandSpan = line.querySelector('.command');
                    let i = 0;
                    const typing = setInterval(() => {
                        if (i < cmd.text.length) {
                            commandSpan.textContent += cmd.text[i];
                            i++;
                        } else {
                            clearInterval(typing);
                            currentCommand++;
                            setTimeout(typeCommand, cmd.delay);
                        }
                    }, 50);
                } else {
                    line.innerHTML = `<span class="output">${cmd.text}</span>`;
                    currentCommand++;
                    setTimeout(typeCommand, cmd.delay);
                }
                
                terminalDemo.appendChild(line);
                
                // Keep only last 8 lines
                while (terminalDemo.children.length > 8) {
                    terminalDemo.removeChild(terminalDemo.firstChild);
                }
            } else {
                // Reset animation
                setTimeout(() => {
                    terminalDemo.innerHTML = '';
                    currentCommand = 0;
                    typeCommand();
                }, 3000);
            }
        }

        // Start animation after page load
        setTimeout(typeCommand, 2000);
    }

    // Intersection Observer for animations
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);

    // Observe elements for animation
    document.querySelectorAll('.feature-card, .step, .demo-card').forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(el);
    });
});

// Add CSS for animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(100%);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }
    
    @keyframes fadeInUp {
        from {
            opacity: 0;
            transform: translateY(30px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    
    .fade-in-up {
        animation: fadeInUp 0.6s ease-out;
    }
`;
document.head.appendChild(style);
