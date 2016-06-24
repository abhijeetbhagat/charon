
; ModuleID = 'main_mod'
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
@.str = private unnamed_addr constant [5 x i8] c"abhi\00"
%_Unwind_Exception = type { i64, void (i32, %_Unwind_Exception*)*, i64, i64 }

define void @cleanup_fn(i32, %_Unwind_Exception) {
entry:
  ret void
}

declare i8* @malloc(i64)

define i32 @main() {
ifcont:
  %s = alloca i64, align 8
  %e = alloca %_Unwind_Exception*, align 8
  store i64 32, i64* %s, align 8
  %0 = load i64* %s, align 8
  %1 = call i8* @malloc(i64 %0)

  %2 = load i64* %s, align 8
  %call4 = call i8* @memset(i8* %1, i32 0, i64 %2)
  %3 = bitcast i8* %1 to %_Unwind_Exception*
  store %_Unwind_Exception* %3, %_Unwind_Exception** %e 
  %4 = load %_Unwind_Exception** %e
  %5 = getelementptr inbounds %_Unwind_Exception* %4, i32 0, i32 0
  store i64 0, i64* %5
  %6 = load %_Unwind_Exception** %e, align 8
  %7 = call i32 @_Unwind_RaiseException(%_Unwind_Exception* %6)

  %call = call i32 (i8*, ...)* @printf(i8* getelementptr inbounds ([5 x i8]* @.str, i32 0, i32 0))
  ret i32 0
}

declare i32 @_Unwind_RaiseException(%_Unwind_Exception*)

declare i32 @printf(i8*, ...)
declare i8* @memset(i8*, i32, i64)
