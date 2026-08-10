#ifndef DNPM_VALIDATOR_UI_MAINWINDOW_H
#define DNPM_VALIDATOR_UI_MAINWINDOW_H

#include <QMainWindow>
#include <QFileDialog>
#include <QTableWidgetItem>
#include <QComboBox>
#include <QLabel>
#include <QTextBlock>
#include <QMessageBox>

#include <lib.rs.h>

QT_BEGIN_NAMESPACE

namespace Ui
{
    class MainWindow;
}

QT_END_NAMESPACE

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget* parent = nullptr);
    ~MainWindow() override;

private:
    Ui::MainWindow* ui;
    QComboBox* formatSelection;
    QLabel* positionLabel;
    QList<dnpmvalidation::ValidationError> errorList;
    QString filename;
    void markErrors();

private slots:
    void onOpenAction();
    void onSaveAction();
    void onSaveAsAction();
    void onValidateAction();
    void onErrorSelected(int index);
};


#endif //DNPM_VALIDATOR_UI_MAINWINDOW_H
