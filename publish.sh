#!/bin/bash

# Interceptor - GitHub Publication Helper Script
# This script helps prepare the repository for publication to S1BGr0uP

set -e

echo "🎯 Interceptor - GitHub Publication Helper"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if git is initialized
if [ ! -d ".git" ]; then
    echo -e "${YELLOW}⚠️  Git not initialized. Initializing...${NC}"
    git init
    echo -e "${GREEN}✅ Git initialized${NC}"
fi

# Check for uncommitted changes
if [[ -n $(git status -s) ]]; then
    echo -e "${YELLOW}⚠️  You have uncommitted changes${NC}"
    echo ""
    git status -s
    echo ""
    read -p "Do you want to commit all changes? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git add .
        read -p "Enter commit message: " commit_msg
        git commit -m "$commit_msg"
        echo -e "${GREEN}✅ Changes committed${NC}"
    fi
fi

# Check if remote exists
if ! git remote get-url origin &> /dev/null; then
    echo -e "${YELLOW}⚠️  No remote 'origin' configured${NC}"
    read -p "Enter GitHub repository URL (e.g., git@github.com:S1BGr0uP/interceptor.git): " repo_url
    git remote add origin "$repo_url"
    echo -e "${GREEN}✅ Remote 'origin' added${NC}"
fi

# Check current branch
current_branch=$(git branch --show-current)
if [ "$current_branch" != "main" ]; then
    echo -e "${YELLOW}⚠️  Current branch is '$current_branch', not 'main'${NC}"
    read -p "Rename to 'main'? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git branch -M main
        echo -e "${GREEN}✅ Branch renamed to 'main'${NC}"
    fi
fi

# Pre-publication checklist
echo ""
echo -e "${BLUE}📋 Pre-Publication Checklist${NC}"
echo "=============================="
echo ""

checklist=(
    "README.md is complete and professional"
    "All code is tested and working"
    "CHANGELOG.md is up to date"
    "LICENSE file has correct copyright"
    "SECURITY.md is reviewed"
    ".gitignore is configured"
    "Sensitive data removed (tokens, keys, etc.)"
    "Documentation is complete"
)

all_checked=true
for item in "${checklist[@]}"; do
    read -p "✓ $item? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        all_checked=false
    fi
done

if [ "$all_checked" = false ]; then
    echo ""
    echo -e "${RED}❌ Please complete all checklist items before publishing${NC}"
    exit 1
fi

# Build check
echo ""
echo -e "${BLUE}🔨 Running build checks...${NC}"
echo ""

echo "Checking Rust backend..."
if cargo check --quiet; then
    echo -e "${GREEN}✅ Rust backend OK${NC}"
else
    echo -e "${RED}❌ Rust backend has errors${NC}"
    exit 1
fi

echo "Checking Vue frontend..."
cd ui
if npm run build > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Vue frontend OK${NC}"
else
    echo -e "${RED}❌ Vue frontend has errors${NC}"
    exit 1
fi
cd ..

# Create tag
echo ""
read -p "Create release tag? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    read -p "Enter version (e.g., v1.0.0): " version
    git tag -a "$version" -m "Release $version"
    echo -e "${GREEN}✅ Tag '$version' created${NC}"
fi

# Push to GitHub
echo ""
echo -e "${BLUE}🚀 Ready to push to GitHub${NC}"
echo ""
read -p "Push to GitHub now? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push -u origin main
    if git tag -l | grep -q .; then
        git push --tags
    fi
    echo ""
    echo -e "${GREEN}✅ Successfully pushed to GitHub!${NC}"
    echo ""
    echo -e "${BLUE}📝 Next steps:${NC}"
    echo "1. Go to GitHub repository"
    echo "2. Create a release from the tag"
    echo "3. Add release notes from CHANGELOG.md"
    echo "4. Configure repository settings"
    echo "5. Add topics/tags"
    echo ""
    echo -e "${GREEN}🎉 Interceptor is now public on S1BGr0uP!${NC}"
else
    echo ""
    echo -e "${YELLOW}⚠️  Push cancelled. Run this script again when ready.${NC}"
fi

echo ""
echo "Done! 🎯"
