#!/bin/bash
cd llvm-tests/llvm/test/Assembler

echo "Negative test categories:"
echo "========================"
echo -n "Debug info (invalid-di*): "
ls invalid-di*.ll 2>/dev/null | wc -l
echo -n "Uselistorder: "
ls invalid-uselistorder*.ll 2>/dev/null | wc -l
echo -n "Metadata: "
ls invalid-md*.ll invalid-metadata*.ll 2>/dev/null | wc -l  
echo -n "Immarg: "
ls invalid-immarg*.ll 2>/dev/null | wc -l
echo -n "Casts: "
ls invalid_cast*.ll invalid-*cast*.ll 2>/dev/null | wc -l
echo -n "Types: "
ls invalid-*type*.ll 2>/dev/null | wc -l
echo -n "Attributes: "
ls *-parse-error*.ll invalid-*attr*.ll 2>/dev/null | wc -l
